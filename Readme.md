# Audbile Split

Takes a single Audbile `.aax`-file and splits it into one `.mp3`-file per chapter.

**Requirement** This tool uses the `ffprobe` and `ffmpeg` CLI utilities. These must either be in the same directory as the tool or in the `PATH`.

## CLI

Run `audbile_split --help` to see all available parameters.

The only required parameters are `--input` the specify the Adudible file and `--activation_bytes` to encrypt it.

## About AAX files

The `.aax` file-format is the format used by Audible for it's AudioBooks. It's basically an AAC audio stream with an encryption header and probably a watermark.

The Audio-stream has Stereo at 64 kbit/s (1 hour audio = 25 MB). Therfore, the quality is already pretty low, which should still be fine for spoken language. The default quality-setting reflects this low input quality and creates files which about match the original file-size while preserving the quality.

However, the output quality can be adjusted with the `--quality` argument. See the programs help for more information.

## Encryption and Activation Bytes

To decrypt an `.aax` file, it's Activation Bytes are required. These will be the same accross all files downloaded from a single Audible account, so you'll only need to run these steps once.

> **Anti-Piracy Notice**
> 
> Please only use this application for gaining full access to your own audiobooks for archiving/converson/convenience. DeDRMed audiobooks should not be uploaded to open servers, torrents, or other methods of mass distribution. No help will be given to people doing such things. Authors, retailers, and publishers all need to make a living, so that they can continue to produce audiobooks for us to hear, and enjoy. Don’t be a parasite.
> 
> This blurb is borrowed from the https://apprenticealf.wordpress.com/ page.

1. Download an Audiobook `.aax` file from Audible and copy it to a work-directory.

    These can be accessed directly from the Audible site from the Download button on the "My Books" page (under "Library"), <https://www.audible.com.au/lib>

    Alternatively, they can be downloaded via the Audible app and the `.aax` file extracted from their application directory:

    * On Windows 10, get the Audbile App from the Microsoft Store and download an AudioBook. The files are located under `C:\Users\[username]\AppData\Local\Packages\AudibleInc.AudibleforWindowsPhone_xns73kv1ymhp2\LocalState\Content`
    * On Linux: ? (PR accepted!)
    * On MacOSX: ? (PR accepted!)
2. Clone or download https://github.com/inAudible-NG/tables
3. Run `ffprobe <file>.aax` and find `file checksum == 999a6ab8...` in the output.
4. Run `tables/run/rcrack tables/ -h <checksum>` and find `result --- hex: CA8...` in the output
5. The 8-character hexadecimal string are the Activation Bytes. These must be supplied to the tool.
6. Run the tool: `audible_split -a <activation_bytes> -i <file>.aax`