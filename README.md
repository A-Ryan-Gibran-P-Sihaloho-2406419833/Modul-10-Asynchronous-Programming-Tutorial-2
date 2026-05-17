# Eksperimen 2.1: Original code, and how it run

![Screenshot eksekusi 1 server dan 3 client](img/img.png)

**Cara Menjalankan:**
1. Buka satu terminal dan jalankan server dengan perintah `cargo run --bin server`. Server akan mendengarkan di port 2000.
2. Buka beberapa terminal baru sebagai client dan jalankan perintah `cargo run --bin client` di masing-masing terminal.
3. Ketik pesan di terminal client dan tekan Enter.

**Apa yang Terjadi:**
Ketika sebuah teks diketik di salah satu client, `tokio::io::stdin().lines()` akan menangkap input tersebut dan mengirimkannya ke server melalui websocket (`ws_stream.send`). Server yang menerima pesan tersebut di dalam fungsi `handle_connection` akan mengirimkan pesan itu ke *channel* `broadcast` tokio (`bcast_tx.send`). Karena setiap koneksi *client* memiliki *subscriber* (`bcast_rx`) ke *channel* yang sama, server akan secara otomatis menerima pesan dari *channel* tersebut lalu meneruskannya kembali ke seluruh *client* yang terhubung. Itulah sebabnya pesan muncul di layar semua *client*.