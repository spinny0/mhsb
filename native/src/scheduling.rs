impl Gpu {
    fn with_context(mut self, context: egui::Context) -> Self {
        let device = self.rs.device.clone();
        let (tx, rx) = std::sync::mpsc::channel::<wgpu::SubmissionIndex>();
        std::thread::Builder::new()
            .name("MHSB GPU completion".into())
            .spawn(move || {
                while let Ok(submission) = rx.recv() {
                    let _ = device.poll(wgpu::PollType::Wait {
                        submission_index: Some(submission),
                        timeout: Some(std::time::Duration::from_secs(10)),
                    });
                    context.request_repaint();
                }
            })
            .expect("GPU completion worker");
        self.wake_sender = Some(tx);
        self
    }
}
