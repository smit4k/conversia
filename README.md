![ConversiaGithubBanner](assets/ConversiaGithubBanner.png)

# 📁 Conversia

Conversia is a powerful, multi-purpose file utility bot written in Rust using the [serenity](https://github.com/serenity-rs/serenity) and [poise](https://github.com/serenity-rs/poise) frameworks. It provides a wide range of file-related operations, making it an essential tool for managing and processing files directly within Discord.

## Features

- **Document Conversion:** Easily convert documents into multiple formats including PDF, Markdown, HTML, Word, and more!
- **Image Conversion**: Transform images between different formats.
- **Image Resizing**: Resize an image using the high-quality Lanczos3 algorithm
- **File Zip/Unzip**: Zip and unzip files
- **File Encryption/Decryption**: Securely encrypt and decrypt files using the Age encryption standard.
- **File Encoding/Decoding**: Securly encode and decode files with base64 and hex
- **Hash Generation/Verification**: Generate a hash and verify the hash for a file with algorithms SHA-256, SHA-1, MD5, BLAKE3
- **Audio Metadata Extraction**: Extract metadata from MP3 and FLAC files, including title, artist, album, year, and genre.

## Add to your Server

Add Conversia to your server [here](https://discord.com/oauth2/authorize?client_id=1373693356928012328&permissions=274877958144&integration_type=0&scope=applications.commands+bot)

## Installation

> [!IMPORTANT] 
> You will need [pandoc](https://pandoc.org/installing.html) and a LaTeX engine (ex. pdfTeX) installed on your system for converting files


1. Clone the repository:

   ```bash
   git clone https://github.com/smit4k/conversia.git
   cd conversia
   ```

2. Install dependencies:

   ```bash
   cargo build --release
   ```

3. Set up the `.env` file:

   ```env
   discord_token=YOUR_DISCORD_BOT_TOKEN
   ```

4. Run the bot:

   ```bash
   cargo run --release
   ```

## Commands

Conversia supports the following commands:

- `/convert_document`: Convert documents to various formats
- `/convert_image`: Convert images between formats
- `/zip`: Compress files into zip
- `/unzip`: Decompress a file
- `/encrypt`: Encrypt files securely
- `/decrypt`: Decrypt encrypted files
- `/base64_encode`: Encode a file to base64
- `/base64_decode`: Decode a base64 encoded file or string
- `/hex_encode`: Encode a file to hex
- `/hex_decode`: Decode a hex encoded file or string
- `/hash`: Generate a hash for a file
- `/resize_image`: Resize an image
- `/audio_meta`: Extract metadata from MP3 and FLAC files
- `/about`: Learn more about Conversia
- `/help`: Shows all commands of Conversia
- `/ping`: Checks the bot's latency

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests to improve Conversia.

## License

This project is licensed under the [MIT License](LICENSE).

## Legal

[Terms of Service](TERMS_OF_SERVICE.md) <br>
[Privacy Policy](PRIVACY_POLICY.md)
