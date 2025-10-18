import yt_dlp
import sys


def download_audio(url: str, output_file_path: str = "."):
    """Download the audio of a YouTube video as an MP3."""
    ydl_opts = {
        "format": "bestaudio/best",     # highest quality audio only
        "outtmpl": output_file_path,  # save as video title
        "postprocessors": [{
            "key": "FFmpegExtractAudio",
            "preferredcodec": "mp3",
            "preferredquality": "192",
        }],
        "quiet": False,                 # set to True if you want to silence output
        "noplaylist": True,             # only single video, not playlists
    }

    with yt_dlp.YoutubeDL(ydl_opts) as ydl:
        ydl.download([url])
        pass


def say_hello():
    print("helo FANOSH")


if __name__ == "__main__":
    download_audio("https://www.youtube.com/watch?v=0yUIL3-xjkM&list=RDGMEMCMFH2exzjBeE_zAHHJOdxgVM0yUIL3-xjkM&index=1", "output/audio")
