fn main() {
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let mut inhibitor: Option<std::process::Child> = None;

        loop {
            let playing: bool = rx.recv().expect("Cannot receive 'sound playing' event");
            if playing {
                if let Some(Ok(Some(_))) | None = inhibitor.as_mut()
                    .map(|c| c.try_wait())
                {
                    inhibitor = Some(inhibit());
                }
            } else {
                if !sound_playing() {
                    let Some(mut inhibitor) = inhibitor.take() else {
                        continue
                    };
                    inhibitor.kill().expect("Cannot kill `systemd-inhibit` process");
                    let _ = inhibitor.wait();
                }
            }
        }
    });

    loop {
        if sound_playing() {
            tx.send(true).expect("Cannot send 'sound playing' event");
            std::thread::sleep(std::time::Duration::from_secs(60));
        } else {
            std::thread::sleep(std::time::Duration::from_secs(60));
            tx.send(false).expect("Cannot send 'no sound' event")
        }
    }
}

fn inhibit() -> std::process::Child {
    std::process::Command::new("systemd-inhibit")
        .arg("--what=shutdown")
        .arg("--what=sleep")
        .arg("--what=idle")
        .arg("--who=caffeine-pulse")
        .arg("--why='sound is playing'")
        .arg("sleep")
        .arg(format!("{}d", i32::MAX))
        .spawn()
        .expect("Cannot spawn `systemd-inhibit`")
}

fn sound_playing() -> bool {
    let pactl_output = std::process::Command::new("pactl")
        .arg("list")
        .arg("sinks")
        .output()
        .expect("Canot run `pactl list sinks`")
        .stdout;

    let output = String::from_utf8(pactl_output)
        .expect("Non UTF8 `pactl list sinks` output");

    output.contains("State: RUNNING")
}
