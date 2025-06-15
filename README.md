# 📁 Conversia

Conversia is a powerful, multi-purpose file utility bot written in Rust using the [serenity](https://github.com/serenity-rs/serenity) and [poise](https://github.com/serenity-rs/poise) frameworks. It provides a wide range of file-related operations, making it an essential tool for managing and processing files directly within Discord.

## Features

- **Document Conversion**: Convert documents to various formats such as PDF, Markdown, HTML, and Word.
- **Image Conversion**: Transform images between different formats.
- **File Compression/Decompression**: Compress and decompress files with formats like ZIP, TAR.GZ, BZ2, ZST, and LZ4.
- **File Encryption/Decryption**: Securely encrypt and decrypt files using the Age encryption standard.
- **Hash Generation**: Generate a has for a file with algorithms SHA-256, SHA-1, MD5, BLAKE3
- **Audio Metadata Extraction**: Extract metadata from MP3 and FLAC files, including title, artist, album, year, and genre.

## Installation

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

- `/convert_document`: Convert documents to various formats.
- `/convert_image`: Convert images between formats.
- `/compress`: Compress files into different formats.
- `/decompress`: Decompress a file
- `/encrypt`: Encrypt files securely.
- `/decrypt`: Decrypt encrypted files.
- `/hash`: Generate a hash for a file
- `/audio_meta`: Extract metadata from MP3 and FLAC files.
- `/about`: Learn more about Conversia.
- `/help`: Shows all commands of Conversia

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests to improve Conversia.

## License

This project is licensed under the [MIT License](LICENSE).

## Legal

[Terms of Service](TERMS_OF_SERVICE.md) <br>
[Privacy Policy](PRIVACY_POLICY.md)
