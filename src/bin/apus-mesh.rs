use std::sync::mpsc::channel;

#[tokio::main]
async fn main() {
    println!("[APUS Mesh] Запуск P2P-узла v0.75 'Hirenkyaku'...");

    // 1. Запуск веб-сервера APUS в фоновом таске
    tokio::spawn(async {
        apus_core::corp::start_dashboard_bind_all(false, true).await;
    });

    println!("[APUS Mesh] Дашборд доступен по адресу: http://localhost:9090");

    // 2. Инициализация трея под Windows
    #[cfg(target_os = "windows")]
    {
        use tray_item::TrayItem;
        
        if let Ok(mut tray) = TrayItem::new("APUS Mesh", tray_item::IconSource::Default) {
            let (tx, rx) = channel();

            let tx_open = tx.clone();
            let _ = tray.add_menu_item("Открыть APUS Dashboard", move || {
                let _ = tx_open.send("OPEN");
            });

            let tx_quit = tx.clone();
            let _ = tray.add_menu_item("Завершить работу APUS", move || {
                let _ = tx_quit.send("QUIT");
            });

            loop {
                if let Ok(msg) = rx.try_recv() {
                    match msg {
                        "OPEN" => { let _ = open::that("http://localhost:9090"); }
                        "QUIT" => { std::process::exit(0); }
                        _ => {}
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }

    // 3. Режим ожидания для Linux / Headless
    #[cfg(not(target_os = "windows"))]
    {
        let _ = open::that("http://localhost:9090");
        tokio::signal::ctrl_c().await.unwrap();
        println!("[APUS Mesh] Завершение работы...");
    }
}
