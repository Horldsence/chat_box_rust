import logging
from vosk import Model, KaldiRecognizer
import pyaudio

class VoskRecognizer:
    def __init__(self, model_path="model/vosk-model-small-cn-0.22"):
        self.model = Model(model_path)
        self.recognizer = KaldiRecognizer(self.model, 16000)
        self.mic = None
        self.stream = None

    def start_stream(self):
        self.mic = pyaudio.PyAudio()
        self.stream = self.mic.open(
            format=pyaudio.paInt16,
            channels=1,
            rate=16000,
            input=True,
            frames_per_buffer=8192
        )

    def stop_stream(self):
        if self.stream:
            self.stream.stop_stream()
            self.stream.close()
        if self.mic:
            self.mic.terminate()

    def recognize_loop(self):
        """持续生成识别结果的生成器"""
        while self.stream and self.stream.is_active():
            data = self.stream.read(4096, exception_on_overflow=False)
            if len(data) == 0:
                break
            if self.recognizer.AcceptWaveform(data):
                yield self.recognizer.Result()
            else:
                yield self.recognizer.PartialResult()

if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    recognizer = VoskRecognizer()
    recognizer.start_stream()
    try:
        for result in recognizer.recognize_loop():
            print(result)
    except KeyboardInterrupt:
        pass
    finally:
        recognizer.stop_stream()
        logging.info("Stream stopped.")