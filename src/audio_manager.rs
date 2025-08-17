use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::{fs::File, io::BufReader, sync::Arc};

pub struct AudioManager {
    _stream: Option<OutputStream>,        // Mantiene vivo el stream si hay audio
    handle: Option<OutputStreamHandle>,   // Controlador si hay audio
    music_sink: Option<Arc<Sink>>,
}

impl AudioManager {
    pub fn new() -> Self {
    match OutputStream::try_default() {
        Ok((stream, handle)) => Self {
            _stream: Some(stream),
            handle: Some(handle),
            music_sink: None,
        },
        Err(_) => {
            println!("No se encontró dispositivo de audio, se desactiva sonido.");
            Self {
                _stream: None,
                handle: None,
                music_sink: None,
            }
        }
    }
}

    /// Reproduce música de fondo en loop
    pub fn play_music(&mut self, path: &str) {
        if self.handle.is_none() {
            return;
        }

        if let Some(sink) = &self.music_sink {
            sink.stop();
        }

        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => {
                println!("No se encontró el archivo de audio: {}", path);
                return;
            }
        };
        let source = match Decoder::new(BufReader::new(file)) {
            Ok(s) => s.repeat_infinite(),
            Err(_) => {
                println!("Error decodificando el audio: {}", path);
                return;
            }
        };

        let sink = Sink::try_new(self.handle.as_ref().unwrap()).unwrap();
        sink.append(source);
        sink.play();
        self.music_sink = Some(Arc::new(sink));
    }

    /// Reproduce efecto de sonido una vez
    pub fn play_sound(&self, path: &str) {
        if self.handle.is_none() {
            return;
        }

        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return,
        };
        let source = match Decoder::new(BufReader::new(file)) {
            Ok(s) => s,
            Err(_) => return,
        };

        let sink = Sink::try_new(self.handle.as_ref().unwrap()).unwrap();
        sink.append(source);
        sink.detach();
    }

    pub fn pause_music(&self) {
        if let Some(sink) = &self.music_sink {
            sink.pause();
        }
    }

    pub fn resume_music(&self) {
        if let Some(sink) = &self.music_sink {
            sink.play();
        }
    }

    pub fn stop_music(&mut self) {
        if let Some(sink) = &self.music_sink {
            sink.stop();
        }
        self.music_sink = None;
    }
}
