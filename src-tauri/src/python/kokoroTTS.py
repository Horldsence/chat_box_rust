from kokoro import KPipeline
import soundfile as sf
import asyncio
from typing import AsyncGenerator, Tuple, Any

class KokoroTTS:
    def __init__(self, lang_code: str = 'a'):
        self.pipeline = KPipeline(lang_code=lang_code)

    async def generate_speech(self, text: str, voice: str = 'af_heart') -> AsyncGenerator[Tuple[int, Any, Any, Any], None]:
        """
        异步生成语音
        Args:
            text: 要转换的文本
            voice: 语音类型，默认为'af_heart'
        Yields:
            Tuple[int, Any, Any, Any]: 包含索引、gs、ps和音频数据的元组
        """
        # 在事件循环中运行同步的生成器
        loop = asyncio.get_event_loop()
        generator = await loop.run_in_executor(None, lambda: self.pipeline(text, voice=voice))

        for i, (gs, ps, audio) in enumerate(generator):
            yield i, gs, ps, audio

    async def save_audio(self, audio: Any, filename: str, sample_rate: int = 24000) -> None:
        """
        异步保存音频文件

        Args:
            audio: 音频数据
            filename: 保存的文件名
            sample_rate: 采样率，默认24000
        """
        loop = asyncio.get_event_loop()
        await loop.run_in_executor(None, lambda: sf.write(filename, audio, sample_rate))

# 使用示例
async def main():
    tts = KokoroTTS()
    text = '''
    [Kokoro](/kˈOkəɹO/) is an open-weight TTS model with 82 million parameters. 
    Despite its lightweight architecture, it delivers comparable quality to larger models 
    while being significantly faster and more cost-efficient. With Apache-licensed weights, 
    [Kokoro](/kˈOkəɹO/) can be deployed anywhere from production environments to personal projects.
    '''

    async for i, gs, ps, audio in tts.generate_speech(text):
        print(f"Processing segment {i}")
        await tts.save_audio(audio, f'{i}.wav')

if __name__ == "__main__":
    asyncio.run(main())