use audio_samples::{AudioProcessing, AudioSamples, AudioStatistics, StandardSample};

pub fn sound_pressure_level<T: StandardSample>(audio: &AudioSamples<'_, T>) -> f64 {
    let rms = audio.rms();
    20.0 * rms.log10()
}

pub fn scale_to_spl<T: StandardSample>(audio: AudioSamples<'_, T>, target_spl: f64) -> AudioSamples<'_, T> {
    let current_spl = sound_pressure_level(&audio);
    let gain_db = target_spl - current_spl;
    let gain_linear = 10.0_f64.powf(gain_db / 20.0);
    audio.scale(gain_linear)
}