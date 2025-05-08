import asyncio
import json
import queue
import threading
import time
import vosk
import pyaudio
import logging
import sys

# 配置日志
logging.basicConfig(level=logging.DEBUG, 
                   format='[%(asctime)s | %(levelname)s | VoskPy] %(message)s',
                   handlers=[logging.StreamHandler(sys.stderr)])
logger = logging.getLogger(__name__)

class VoskRecognizer:
    def __init__(self, model_path=None):
        logger.info(f"初始化VoskRecognizer，模型路径：{model_path}")
        if model_path:
            self.model = vosk.Model(model_path)
        else:
            # 默认模型路径
            self.model = vosk.Model("model/vosk-model-small-zh-cn")

        self.recognizer = vosk.KaldiRecognizer(self.model, 16000)
        self.audio = pyaudio.PyAudio()
        self.stream = None
        self.running = False
        self.result_queue = queue.Queue()
        self._recognition_thread = None
        self._lock = threading.Lock()  # 添加锁以保护关键区域
        logger.info("VoskRecognizer初始化完成")

    def start_stream(self):
        """启动音频流但不开始识别"""
        with self._lock:
            if self.stream:
                logger.debug("音频流已存在，不再重新创建")
                return

            logger.debug("创建新的音频流")
            try:
                self.stream = self.audio.open(
                    format=pyaudio.paInt16,
                    channels=1,
                    rate=16000,
                    input=True,
                    frames_per_buffer=8000
                )
                self.running = True
                logger.info("音频流启动成功")
            except Exception as e:
                logger.error(f"启动音频流失败: {e}")
                raise

    def stop_stream(self):
        """停止音频流"""
        with self._lock:
            self.running = False
            if self.stream:
                logger.debug("正在关闭音频流")
                try:
                    self.stream.stop_stream()
                    self.stream.close()
                    self.stream = None
                    logger.info("音频流关闭成功")
                except Exception as e:
                    logger.error(f"关闭音频流时出错: {e}")

    def start_recognition(self, timeout_seconds=30.0, end_on_silence=True):
        """启动识别过程，结果放入队列"""
        logger.info(f"开始语音识别，超时时间: {timeout_seconds}秒，静默检测: {end_on_silence}")
        self.start_stream()
        
        # 清空现有队列
        while not self.result_queue.empty():
            self.result_queue.get_nowait()

        # 确保之前的线程已结束
        if self._recognition_thread and self._recognition_thread.is_alive():
            logger.warning("已有识别线程正在运行，等待其结束")
            self.running = False
            self._recognition_thread.join(2.0)  # 等待最多2秒

        self.running = True

        def recognition_thread():
            start_time = time.time()
            silence_threshold = 3.0  # 3秒无声判定为结束
            last_active = time.time()
            silence_detected = False
            
            logger.debug("识别线程已启动")
            try:
                while self.running and (time.time() - start_time) < timeout_seconds:
                    if not self.stream or not self.running:
                        logger.warning("流已关闭或运行标志已关闭，结束识别")
                        break
                        
                    try:
                        data = self.stream.read(4000, exception_on_overflow=False)
                        
                        # 检查是否有声音输入（计算音频振幅）
                        if len(data) >= 2:  # 确保有足够的数据
                            amplitude = max(abs(int.from_bytes(data[i:i+2], byteorder='little', signed=True)) 
                                         for i in range(0, len(data), 2))
                            is_silent = amplitude < 500
                            
                            if not is_silent:
                                last_active = time.time()
                                silence_detected = False
                            elif time.time() - last_active > silence_threshold and end_on_silence and not silence_detected:
                                logger.info(f"检测到{silence_threshold}秒静默")
                                self.result_queue.put("[silence detected]")
                                silence_detected = True
                                if end_on_silence:
                                    break

                        # 处理语音识别
                        if self.recognizer.AcceptWaveform(data):
                            result = self.recognizer.Result()
                            text = json.loads(result)["text"]
                            if text.strip():
                                logger.debug(f"识别结果: {text}")
                                self.result_queue.put(text)
                        else:
                            partial = self.recognizer.PartialResult()
                            text = json.loads(partial)["partial"]
                            if text.strip():
                                logger.debug(f"部分结果: {text}")
                                self.result_queue.put(text)
                    except Exception as e:
                        logger.error(f"识别过程中出错: {e}")
                        self.result_queue.put(f"[error] {str(e)}")
                        break

                # 检查超时
                if (time.time() - start_time) >= timeout_seconds:
                    logger.info(f"达到超时时间 {timeout_seconds}秒")
                    self.result_queue.put("[timeout reached]")

                # 标记结束  
                self.result_queue.put("[end]")
            except Exception as e:
                logger.error(f"识别线程异常: {e}")
                self.result_queue.put(f"[error] {str(e)}")
            finally:
                # 不在这里自动停止流，让调用方决定何时停止
                logger.debug("识别线程退出")
                
        # 保存线程引用并启动
        self._recognition_thread = threading.Thread(target=recognition_thread)
        self._recognition_thread.daemon = True  # 设为守护线程，当主线程退出时自动结束
        self._recognition_thread.start()

    def stop_recognition(self):
        """停止识别过程"""
        logger.info("停止识别过程")
        with self._lock:
            self.running = False
            # 不要在这里停止流，分离流管理和识别过程
            if self._recognition_thread and self._recognition_thread.is_alive():
                logger.debug("等待识别线程结束")
                self._recognition_thread.join(1.0)  # 等待最多1秒

    def get_result(self):
        """获取最新的识别结果，非阻塞"""
        try:
            result = self.result_queue.get_nowait()
            logger.debug(f"获取结果: {result}")
            return result
        except queue.Empty:
            return None

    def __del__(self):
        """析构函数确保资源被释放"""
        logger.info("VoskRecognizer实例被销毁")
        self.running = False
        self.stop_stream()

if __name__ == "__main__":
    recognizer = VoskRecognizer(model_path="C:/Users/18511/Documents/AppCode/Rust/chat_box/src-tauri/model/vosk-model-small-cn-0.22")
    recognizer.start_recognition(timeout_seconds=10.0)

    try:
        while True:
            result = recognizer.get_result()
            if result:
                print(result)
                if result == "[end]":
                    break
    except KeyboardInterrupt:
        recognizer.stop_recognition()