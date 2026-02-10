use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use teloxide::types::File as TelegramFile;
use teloxide::net::Download;
use teloxide::Bot;
use std::fs::File;
use std::io::Write;
use futures_util::StreamExt;

use crate::config::TranscriptionConfig;

#[cfg(feature = "whisper-rs")]
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// Trait for transcription providers
#[async_trait::async_trait]
pub trait TranscriptionProvider: Send + Sync {
    async fn transcribe(&self, bot: &Bot, file: &TelegramFile, temp_dir: &str) -> Result<String>;
}

/// Factory function to create the appropriate transcription provider
pub fn create_transcription_provider(config: &TranscriptionConfig) -> Result<Box<dyn TranscriptionProvider>> {
    match config.provider.as_str() {
        "whisper_local" => {
            let model_path = config.model_path.as_deref()
                .context("model_path is required for whisper_local provider")?;
            Ok(Box::new(WhisperLocalProvider {
                model_path: model_path.to_string(),
                language: config.language.clone(),
            }))
        }
        "groq" => {
            let api_key_env = config.api_key_env.as_deref()
                .unwrap_or("GROQ_API_KEY");
            let api_key = std::env::var(api_key_env)
                .with_context(|| format!("Environment variable '{}' not set. Required for Groq provider.", api_key_env))?;
            let model = config.model.as_deref()
                .unwrap_or("whisper-large-v3-turbo")
                .to_string();
            Ok(Box::new(GroqProvider {
                api_key,
                model,
                language: config.language.clone(),
            }))
        }
        other => anyhow::bail!("Unknown transcription provider: '{}'. Use 'whisper_local' or 'groq'.", other),
    }
}

// ---------------------------------------------------------------------------
// WhisperLocalProvider
// ---------------------------------------------------------------------------

pub struct WhisperLocalProvider {
    model_path: String,
    language: String,
}

#[async_trait::async_trait]
impl TranscriptionProvider for WhisperLocalProvider {
    async fn transcribe(&self, bot: &Bot, file: &TelegramFile, temp_dir: &str) -> Result<String> {
        // Download audio from Telegram
        let audio_path = download_audio_file(bot, file, temp_dir).await?;

        // Convert to WAV format
        let wav_path = convert_audio_to_wav(&audio_path)
            .context("Failed to convert audio to WAV")?;

        // Transcribe
        let transcript = transcribe_with_whisper(&wav_path, &self.model_path, &self.language)?;

        // Clean up temporary files
        if let Err(e) = std::fs::remove_file(&audio_path) {
            log::warn!("Failed to remove temporary audio file: {}", e);
        }
        if let Err(e) = std::fs::remove_file(&wav_path) {
            log::warn!("Failed to remove temporary WAV file: {}", e);
        }

        Ok(transcript)
    }
}

// ---------------------------------------------------------------------------
// GroqProvider
// ---------------------------------------------------------------------------

pub struct GroqProvider {
    api_key: String,
    model: String,
    language: String,
}

#[async_trait::async_trait]
impl TranscriptionProvider for GroqProvider {
    async fn transcribe(&self, bot: &Bot, file: &TelegramFile, temp_dir: &str) -> Result<String> {
        // Download audio from Telegram (keep as OGG — Groq accepts it)
        let audio_path = download_audio_file(bot, file, temp_dir).await?;

        let file_bytes = std::fs::read(&audio_path)
            .context("Failed to read downloaded audio file")?;

        let file_name = audio_path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let file_part = reqwest::multipart::Part::bytes(file_bytes)
            .file_name(file_name)
            .mime_str("audio/ogg")?;

        let form = reqwest::multipart::Form::new()
            .part("file", file_part)
            .text("model", self.model.clone())
            .text("language", self.language.clone())
            .text("response_format", "json");

        let client = reqwest::Client::new();
        let response = client
            .post("https://api.groq.com/openai/v1/audio/transcriptions")
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .await
            .context("Failed to send request to Groq API")?;

        // Clean up temp file
        if let Err(e) = std::fs::remove_file(&audio_path) {
            log::warn!("Failed to remove temporary audio file: {}", e);
        }

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Groq API error ({}): {}", status, error_text);
        }

        let response_json: serde_json::Value = response.json().await
            .context("Failed to parse Groq response")?;

        let text = response_json["text"]
            .as_str()
            .context("No 'text' field in Groq response")?
            .to_string();

        log::info!("Groq transcription complete: {} characters", text.len());
        Ok(text)
    }
}

// ---------------------------------------------------------------------------
// Shared helpers (download, convert, whisper)
// ---------------------------------------------------------------------------

/// Download audio file from Telegram
async fn download_audio_file(
    bot: &Bot,
    file: &TelegramFile,
    temp_dir: &str,
) -> Result<PathBuf> {
    log::info!("Downloading audio file: {}", file.path);

    // Create temp directory if it doesn't exist
    std::fs::create_dir_all(temp_dir)?;

    // Generate unique filename
    let file_name = format!("audio_{}.ogg", uuid::Uuid::new_v4());
    let file_path = Path::new(temp_dir).join(&file_name);

    // Download file from Telegram
    let mut stream = bot.download_file_stream(&file.path);
    let mut dest_file = File::create(&file_path)
        .context("Failed to create temporary audio file")?;

    // Write chunks to file
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Failed to download audio chunk")?;
        dest_file.write_all(&chunk)
            .context("Failed to write audio chunk to file")?;
    }

    log::info!("Audio file downloaded to: {}", file_path.display());
    Ok(file_path)
}

/// Convert audio using ffmpeg (fallback for unsupported formats like Opus)
fn convert_with_ffmpeg(input_path: &Path, output_path: &Path) -> Result<()> {
    use std::process::Command;

    log::info!("Converting audio with ffmpeg...");

    let output = Command::new("ffmpeg")
        .arg("-i")
        .arg(input_path)
        .arg("-ar")
        .arg("16000") // 16kHz sample rate
        .arg("-ac")
        .arg("1") // mono
        .arg("-c:a")
        .arg("pcm_s16le") // 16-bit PCM
        .arg("-y") // overwrite output
        .arg(output_path)
        .output()
        .context("Failed to run ffmpeg. Is ffmpeg installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg conversion failed: {}", stderr);
    }

    log::info!("Audio converted successfully with ffmpeg");
    Ok(())
}

/// Convert audio file to WAV format (16kHz, mono) for Whisper
fn convert_audio_to_wav(input_path: &Path) -> Result<PathBuf> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    log::info!("Converting audio to WAV format: {}", input_path.display());

    // Check if file exists and has content
    let metadata = std::fs::metadata(input_path)
        .context("Failed to read audio file metadata")?;
    log::info!("Audio file size: {} bytes", metadata.len());

    if metadata.len() == 0 {
        anyhow::bail!("Audio file is empty");
    }

    // Open the media source
    let src = std::fs::File::open(input_path)
        .context("Failed to open audio file")?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    // Create a probe hint
    let mut hint = Hint::new();
    if let Some(ext) = input_path.extension() {
        let extension = ext.to_string_lossy();
        log::info!("Audio file extension: {}", extension);
        hint.with_extension(&extension);
    } else {
        log::warn!("Audio file has no extension, probing without hint");
    }

    // Probe the media source
    log::info!("Probing audio format...");
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .context("Failed to probe audio file. The audio format may not be supported.")?;

    log::info!("Audio format detected successfully");

    let mut format = probed.format;

    // Find the first audio track and extract codec params
    log::info!("Finding audio track...");
    let track = format.tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .context("No audio track found in file")?;

    let track_id = track.id;
    let codec_params = track.codec_params.clone();
    log::info!("Audio track found with codec type: {:?}", codec_params.codec);

    // Get sample rate and channels before consuming format
    let sample_rate = codec_params.sample_rate
        .context("Sample rate not found")?;
    let channels = codec_params.channels
        .context("Channel info not found")?
        .count();

    // Create a decoder for the track
    log::info!("Creating decoder for codec...");
    let decoder_result = symphonia::default::get_codecs()
        .make(&codec_params, &DecoderOptions::default());

    // If Symphonia can't decode (e.g., Opus codec), fall back to ffmpeg
    if decoder_result.is_err() {
        log::warn!("Symphonia can't decode this format. Falling back to ffmpeg...");
        let output_path = input_path.with_extension("wav");
        convert_with_ffmpeg(input_path, &output_path)?;
        return Ok(output_path);
    }

    let mut decoder = decoder_result.context("Failed to create decoder")?;
    log::info!("Decoder created successfully");

    // Decode and collect samples
    let mut samples = Vec::new();
    let mut sample_buf = None;

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e).context("Failed to read packet")?,
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = decoder.decode(&packet)
            .context("Failed to decode packet")?;

        if sample_buf.is_none() {
            let spec = *decoded.spec();
            let duration = decoded.capacity() as u64;
            sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
        }

        if let Some(ref mut buf) = sample_buf {
            buf.copy_interleaved_ref(decoded);
            samples.extend_from_slice(buf.samples());
        }
    }

    if samples.is_empty() {
        anyhow::bail!("No audio samples decoded");
    }

    log::info!("Original audio: {} Hz, {} channels, {} samples",
               sample_rate, channels, samples.len());

    // Convert to mono if stereo
    let mono_samples = if channels > 1 {
        samples.chunks(channels)
            .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
            .collect::<Vec<f32>>()
    } else {
        samples
    };

    // Resample to 16kHz if needed
    let target_sample_rate = 16000;
    let resampled = if sample_rate != target_sample_rate {
        resample_audio(&mono_samples, sample_rate, target_sample_rate)
    } else {
        mono_samples
    };

    // Convert f32 samples to i16 for WAV
    let i16_samples: Vec<i16> = resampled.iter()
        .map(|&s| (s.clamp(-1.0, 1.0) * 32767.0) as i16)
        .collect();

    // Write WAV file
    let output_path = input_path.with_extension("wav");
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: target_sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(&output_path, spec)
        .context("Failed to create WAV writer")?;

    for sample in i16_samples {
        writer.write_sample(sample)
            .context("Failed to write WAV sample")?;
    }

    writer.finalize()
        .context("Failed to finalize WAV file")?;

    log::info!("Audio converted to: {}", output_path.display());
    Ok(output_path)
}

/// Simple linear resampling (for better quality, consider using a proper resampling library)
fn resample_audio(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return samples.to_vec();
    }

    let ratio = from_rate as f64 / to_rate as f64;
    let output_len = (samples.len() as f64 / ratio) as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_idx = (i as f64 * ratio) as usize;
        if src_idx < samples.len() {
            output.push(samples[src_idx]);
        }
    }

    log::info!("Resampled from {} Hz to {} Hz", from_rate, to_rate);
    output
}

/// Transcribe audio file using Whisper
#[cfg(feature = "whisper-rs")]
fn transcribe_with_whisper(wav_path: &Path, model_path: &str, language: &str) -> Result<String> {
    log::info!("Transcribing audio with Whisper model: {}", model_path);

    // Load Whisper model
    let ctx = WhisperContext::new_with_params(
        model_path,
        WhisperContextParameters::default(),
    ).context("Failed to load Whisper model")?;

    // Load audio data
    let mut reader = hound::WavReader::open(wav_path)
        .context("Failed to open WAV file")?;

    let audio_data: Vec<f32> = reader.samples::<i16>()
        .map(|s| s.unwrap() as f32 / 32768.0)
        .collect();

    log::info!("Audio loaded: {} samples", audio_data.len());

    // Create transcription state
    let mut state = ctx.create_state()
        .context("Failed to create Whisper state")?;

    // Configure transcription parameters
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some(language));
    params.set_print_progress(false);
    params.set_print_special(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    // Run transcription
    state.full(params, &audio_data)
        .context("Failed to run Whisper transcription")?;

    // Extract transcribed text
    let num_segments = state.full_n_segments()
        .context("Failed to get number of segments")?;

    let mut transcript = String::new();
    for i in 0..num_segments {
        let segment = state.full_get_segment_text(i)
            .context("Failed to get segment text")?;
        transcript.push_str(&segment);
        transcript.push(' ');
    }

    let transcript = transcript.trim().to_string();
    log::info!("Transcription complete: {} characters", transcript.len());

    Ok(transcript)
}

#[cfg(not(feature = "whisper-rs"))]
fn transcribe_with_whisper(_wav_path: &Path, _model_path: &str, _language: &str) -> Result<String> {
    anyhow::bail!("Whisper feature not enabled. Build with --features metal (Mac) or --features cuda (Windows)")
}
