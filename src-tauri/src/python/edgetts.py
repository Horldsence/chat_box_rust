import edge_tts

class TextToSpeech:
    def __init__(self, voice="zh-CN-XiaoxiaoNeural"):
        self.voice = voice
        self.tts = self.voice

    async def text_to_speech(self, text, output_file="output.mp3"):
        """
        Convert text to speech and save it to an audio file.

        :param text: The text to convert to speech.
        :param output_file: The file path to save the audio.
        """
        communicate = edge_tts.Communicate(text, voice=self.voice, rate="+0%")
        with open(output_file, "wb") as file:
            async for chunk in communicate.stream():
                if chunk["type"] == "audio":
                    file.write(chunk["data"])

    def text_to_speech_sync(self, text, output_file="output.mp3"):
        import asyncio
        asyncio.run(self.text_to_speech(text, output_file))

if __name__ == "__main__":
    import asyncio
    import argparse

    parser = argparse.ArgumentParser(description="Convert text to speech using Edge TTS.")
    parser.add_argument("--text", required=True, help="Text to convert")
    parser.add_argument("--output", default="output.mp3", help="Output file path")
    args = parser.parse_args()

    tts = TextToSpeech()
    asyncio.run(tts.text_to_speech(args.text, args.output))