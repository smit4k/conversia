use crate::{Context, Error};
use ::serenity::all::{ButtonStyle, CreateActionRow, CreateButton};
use poise::serenity_prelude as serenity;

#[derive(Clone, Copy)]
enum HelpPage {
    Overview,
    Conversion,
    Encryption,
    Encoding,
    Compression,
    Other,
}

impl HelpPage {
    fn from_custom_id(custom_id: &str) -> Option<Self> {
        match custom_id {
            "help_overview" => Some(Self::Overview),
            "help_conversion" => Some(Self::Conversion),
            "help_encryption" => Some(Self::Encryption),
            "help_encoding" => Some(Self::Encoding),
            "help_compression" => Some(Self::Compression),
            "help_other" => Some(Self::Other),
            _ => None,
        }
    }

    fn create_embed(&self) -> serenity::builder::CreateEmbed {
        match self {
            HelpPage::Overview => {
                serenity::builder::CreateEmbed::default()
                    .title("Conversia Help - Overview")
                    .description(
                        "Welcome to Conversia! A powerful file conversion and utility bot.",
                    )
                    .color(0x5865F2) // Discord blurple
                    .field(
                        "📄 Conversion",
                        "Convert documents and images between formats",
                        false,
                    )
                    .field(
                        "🔒 Encryption",
                        "Secure your files with encryption/decryption",
                        false,
                    )
                    .field(
                        "📦 Compression",
                        "Compress files into various archive formats",
                        false,
                    )
                    .field(
                        "🛠️ Other Tools",
                        "Additional utilities like metadata extraction",
                        false,
                    )
                    .footer(serenity::builder::CreateEmbedFooter::new(
                        "Use the buttons below to explore each category",
                    ))
            }
            HelpPage::Conversion => {
                serenity::builder::CreateEmbed::default()
                    .title("Conversia Help - Conversion")
                    .description("Convert your files between different formats")
                    .color(0x00D166)  // Green
                    .field("/convert_document", "Convert documents to formats like PDF, Markdown, HTML, and Word.\n• Supports: MD, HTML, PDF, TEX, DOCX, RTF, ODT, EPUB", false)
                    .field("/convert_image", "Convert images between formats.\n• Supports: JPG, PNG, WEBP, GIF, BMP, TIFF", false)
                    .footer(serenity::builder::CreateEmbedFooter::new("Choose the format that best fits your needs"))
            }
            HelpPage::Encryption => {
                serenity::builder::CreateEmbed::default()
                    .title("Conversia Help - Encryption")
                    .description("Secure your files with modern encryption")
                    .color(0xFF6B6B)  // Red
                    .field("/encrypt", "Encrypt files securely using the Age encryption standard.\n• Password-based encryption\n• Secure and modern cryptography", false)
                    .field("/decrypt", "Decrypt files that were previously encrypted.\n• Supports Age-encrypted files\n• Requires the original password", false)
                    .footer(serenity::builder::CreateEmbedFooter::new("Save your password! - you'll need it to decrypt your file"))
            }
            HelpPage::Encoding => {
                serenity::builder::CreateEmbed::default()
                    .title("Conversia Help - Encoding")
                    .description("Encode files to different formats")
                    .color(0x3498DB) // Blue
                    .field("/base64_encode", "Encode a file to Base64 format.", false)
                    .field("/base64_decode", "Decode a Base64-encoded file.", false)
                    .field("/hex_encode", "Encode a file to Hex format.", false)
                    .field("/hex_decode", "Decode a Hex-encoded file.", false)
                    .footer(serenity::builder::CreateEmbedFooter::new(
                        "Encoding is useful for data transfer and storage",
                    ))
            }
            HelpPage::Compression => {
                serenity::builder::CreateEmbed::default()
                    .title("Conversia Help - Compression")
                    .description("Compress files to save space and organize")
                    .color(0xFFD93D)  // Yellow
                    .field("/zip", "Compress a file into a ZIP archive.", false)
                    .field("/unzip", "Extract a single file from a ZIP archive.\n• Multi-file ZIPs are currently rejected", false)
                    .footer(serenity::builder::CreateEmbedFooter::new("ZIP extraction currently supports single-file archives"))
            }
            HelpPage::Other => {
                serenity::builder::CreateEmbed::default()
                    .title("Conversia Help - Other Tools")
                    .description("Additional utilities and information")
                    .color(0x9B59B6)  // Purple
                    .field("/resize_image", "Resize an image", false)
                    .field("/hash", "Generate a hash for a file.\n• Supports: SHA-256, SHA-1, MD5, BLAKE3", false)
                    .field("/verify_hash", "Verify a file’s hash against an expected checksum \n• Supports: SHA-256, SHA-1, MD5, BLAKE3", false)
                    .field("/audio_meta", "Extract metadata from MP3 and FLAC files.\n• Shows: title, artist, album, year, genre\n• Works with most MP3 and FLAC files", false)
                    .field("/about", "Learn more about the Conversia bot.\n• Information about the bot\n• Legal information", false)
                    .field("/ping", "Check the bot's latency.\n• Useful for debugging connection issues", false)
                    .field("/help", "Shows this help system.\n• Navigate between categories\n• Find detailed command information", false)
                    .footer(serenity::builder::CreateEmbedFooter::new("More tools coming soon!"))
            }
        }
    }

    fn create_buttons(&self) -> Vec<CreateButton> {
        let mut buttons = vec![
            CreateButton::new("help_overview")
                .label("Overview")
                .style(if matches!(self, HelpPage::Overview) {
                    ButtonStyle::Primary
                } else {
                    ButtonStyle::Secondary
                })
                .emoji('🏠'),
            CreateButton::new("help_conversion")
                .label("Conversion")
                .style(if matches!(self, HelpPage::Conversion) {
                    ButtonStyle::Primary
                } else {
                    ButtonStyle::Secondary
                })
                .emoji('📄'),
            CreateButton::new("help_encryption")
                .label("Encryption")
                .style(if matches!(self, HelpPage::Encryption) {
                    ButtonStyle::Primary
                } else {
                    ButtonStyle::Secondary
                })
                .emoji('🔒'),
            CreateButton::new("help_encoding")
                .label("Encoding")
                .style(if matches!(self, HelpPage::Encoding) {
                    ButtonStyle::Primary
                } else {
                    ButtonStyle::Secondary
                })
                .emoji('🔤'),
            CreateButton::new("help_compression")
                .label("Compression")
                .style(if matches!(self, HelpPage::Compression) {
                    ButtonStyle::Primary
                } else {
                    ButtonStyle::Secondary
                })
                .emoji('📦'),
            CreateButton::new("help_other")
                .label("Other")
                .style(if matches!(self, HelpPage::Other) {
                    ButtonStyle::Primary
                } else {
                    ButtonStyle::Secondary
                })
                .emoji('🛠'),
        ];

        // Add GitHub issue button as a link button
        buttons.push(
            CreateButton::new_link("https://github.com/smit4k/conversia/issues")
                .label("Report Bug")
                .emoji('🔗'),
        );

        buttons
    }
}

fn help_action_rows(buttons: Vec<CreateButton>) -> Vec<CreateActionRow> {
    if buttons.len() <= 5 {
        vec![CreateActionRow::Buttons(buttons)]
    } else {
        let (first_row, second_row) = buttons.split_at(5);
        vec![
            CreateActionRow::Buttons(first_row.to_vec()),
            CreateActionRow::Buttons(second_row.to_vec()),
        ]
    }
}

fn help_response(page: HelpPage) -> poise::CreateReply {
    poise::CreateReply::default()
        .embed(page.create_embed())
        .components(help_action_rows(page.create_buttons()))
}

/// Shows all commands available
#[poise::command(slash_command, prefix_command)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    let message = ctx.send(help_response(HelpPage::Overview)).await?;

    // Handle button interactions
    let message = message.into_message().await?;
    let collector = message
        .await_component_interactions(ctx.serenity_context())
        .timeout(std::time::Duration::from_secs(300)) // 5 minutes timeout
        .stream();

    use serenity::futures::StreamExt;
    let mut collector = collector;

    while let Some(interaction) = collector.next().await {
        let Some(new_page) = HelpPage::from_custom_id(&interaction.data.custom_id) else {
            continue;
        };

        let edit_response = serenity::builder::CreateInteractionResponse::UpdateMessage(
            serenity::builder::CreateInteractionResponseMessage::default()
                .embed(new_page.create_embed())
                .components(help_action_rows(new_page.create_buttons())),
        );

        if let Err(e) = interaction
            .create_response(&ctx.http(), edit_response)
            .await
        {
            eprintln!("Error updating help message: {}", e);
        }
    }

    Ok(())
}
