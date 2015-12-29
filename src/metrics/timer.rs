use super::MetricsRecorder;


pub struct Timer<'a> {
    recorder: &'a mut MetricsRecorder,
    name: String,
}

impl<'a> Timer<'a> {
    pub fn new(recorder: &'a mut MetricsRecorder, name: &str) -> Timer<'a> {
        recorder.start_timer(name);

        Timer {
            name: name.to_string(),
            recorder: recorder,
        }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        self.recorder.stop_timer(&self.name);
    }
}
