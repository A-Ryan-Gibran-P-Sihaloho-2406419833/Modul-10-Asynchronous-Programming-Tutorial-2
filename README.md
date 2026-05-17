# Eksperimen 2.1: Original code, and how it run

![Screenshot eksekusi 1 server dan 3 client](img/img.png)

**Cara Menjalankan:**
1. Buka satu terminal dan jalankan server dengan perintah `cargo run --bin server`. Server akan mendengarkan di port 2000.
2. Buka beberapa terminal baru sebagai client dan jalankan perintah `cargo run --bin client` di masing-masing terminal.
3. Ketik pesan di terminal client dan tekan Enter.

**Apa yang Terjadi:**
Ketika sebuah teks diketik di salah satu client, `tokio::io::stdin().lines()` akan menangkap input tersebut dan mengirimkannya ke server melalui websocket (`ws_stream.send`). Server yang menerima pesan tersebut di dalam fungsi `handle_connection` akan mengirimkan pesan itu ke *channel* `broadcast` tokio (`bcast_tx.send`). Karena setiap koneksi *client* memiliki *subscriber* (`bcast_rx`) ke *channel* yang sama, server akan secara otomatis menerima pesan dari *channel* tersebut lalu meneruskannya kembali ke seluruh *client* yang terhubung. Itulah sebabnya pesan muncul di layar semua *client*.


# Eksperimen 2.2: Modifying port
![Screenshot eksekusi 1 server dan 3 client dengan port 8080](img/img_1.png)
**Perubahan yang Dilakukan:**
Untuk mengubah port menjadi 8080, modifikasi dilakukan pada dua sisi karena koneksi WebSocket membutuhkan kecocokan antara alamat yang didengarkan (server) dan alamat yang dituju (client):
1. **Server (`src/bin/server.rs`)**: Mengubah argumen pada `TcpListener::bind("127.0.0.1:8080")` agar server mendengarkan koneksi masuk di port 8080.
2. **Client (`src/bin/client.rs`)**: Mengubah URI pada `ClientBuilder::from_uri(Uri::from_static("ws://127.0.0.1:8080"))` agar client mencoba menyambung ke port 8080.

**Protokol Websocket:**
Ya, aplikasi masih menggunakan protokol websocket yang sama. Protokol ini didefinisikan dengan skema `ws://` (yang merupakan singkatan dari WebSocket over HTTP) pada *string* URI di file client saat memanggil `Uri::from_static("ws://127.0.0.1:8080")`. Sedangkan di sisi server, protokol ini dikelola oleh fungsi `ServerBuilder::new().accept(socket)` yang melakukan proses *handshake* untuk meng- *upgrade* koneksi TCP biasa menjadi koneksi WebSocket.

# Eksperimen 2.3: Small changes. Add some information to client

![Screenshot eksekusi penambahan IP dan Port](img/img_2.png)

**Penjelasan Perubahan:**
Untuk menambahkan informasi IP dan Port pengirim, saya melakukan modifikasi pada file `src/bin/server.rs`. Di dalam fungsi `handle_connection`, ketika server menerima pesan teks dari suatu client, server tidak lagi langsung mem-broadcast `text` mentah.

Sebagai gantinya, saya memformat pesan baru menggunakan `format!("{addr}: {text}")`, di mana `addr` adalah `SocketAddr` (IP dan Port) milik client pengirim yang didapat saat koneksi pertama kali diterima. Pesan yang sudah diformat inilah yang kemudian dikirim ke `bcast_tx` untuk disiarkan ke seluruh client.

# Tugas Bonus: Implementasi Asynchronous WebSocket Server Berbasis Rust untuk YewChat Client

Bagian ini menjelaskan keberhasilan implementasi Tugas Bonus, yaitu menggantikan server WebSocket bawaan Node.js dari Tutorial 3 menggunakan server asinkronus berbasis Rust yang terintegrasi di dalam repositori ini.

## 📝 Apa yang Sudah Dilakukan?

Kita telah membuat sebuah berkas biner eksekusi (*binary executable*) baru bernama `src/bin/server_bonus.rs`. Berkas ini memodifikasi arsitektur dasar server multitafsir teks mentah pada Tutorial 2 agar mampu melakukan serialisasi dan deserialisasi data terstruktur **JSON** yang dikirimkan oleh aplikasi klien *frontend* Yew (Tutorial 3).

Server baru ini menangani beberapa fungsionalitas kritis:
1. **Pendaftaran Pengguna (*Client Registration*):** Menangkap tipe pesan `register` untuk menyimpan pemetaan antara alamat soket TCP (`SocketAddr`) klien dengan *username* pilihan mereka secara aman menggunakan *state* terbagi (`Arc<Mutex<HashMap<...>>>`).
2. **Sinkronisasi Daftar Pengguna Online:** Mem-broadcast array nama seluruh pengguna yang aktif ke semua klien terhubung setiap kali ada pengguna yang bergabung atau memutuskan koneksi (*disconnect*).
3. **Penyebaran Pesan Global (*Message Broadcasting*):** Mengemas ulang kiriman obrolan ke dalam objek `MessageData` struktural sebelum disebarkan ke seluruh klien yang terhubung secara asinkron.

---

## 🔍 Perbedaan Teknis Antar Server

| Aspek Perbandingan | Server Tutorial 3 (Original) | Server Tutorial 2 (Original) | Server Tugas Bonus (Rust) |
| :--- | :--- | :--- | :--- |
| **Teknologi / Bahasa** | Node.js (JavaScript) | Rust (`tokio` + `tokio-tungstenite`) | Rust (`tokio` + `tokio-tungstenite`) |
| **Format Data Jaringan** | JSON Berstruktur Kuat | Teks Mentah (*Raw Plain Text*) | JSON Berstruktur Kuat |
| **Penanganan State Klien** | Array dinamis JavaScript | `HashMap` dilindungi oleh `Arc<Mutex>` | `HashMap` dengan pelacakan data kembar (*Tuple* berisi `Tx` dan *Username*) |
| **Manajemen Memori** | Mengandalkan *Garbage Collector* | *Compile-time safety* (tanpa GC) | *Compile-time safety* dengan jaminan bebas *race condition* |

---

## 🚀 Panduan Menjalankan Aplikasi Secara Penuh (Panduan Asdos)

Untuk menjalankan skenario Tugas Bonus ini, pastikan Anda **TIDAK** menjalankan server Node.js (`tutorial-3-server`) karena port `8080` akan digunakan sepenuhnya oleh server Rust ini.

### Langkah 1: Jalankan Server Rust (Tugas Bonus)
1. Buka terminal di dalam root direktori repositori ini (`tutorial-2-chat`).
2. Eksekusi biner khusus server bonus menggunakan perintah Cargo:
   ```bash
   cargo run --bin server_bonus
   ```
3. Jika berhasil, terminal akan menampilkan log:
   `🦀 Server Bonus (Rust) berjalan di: ws://127.0.0.1:8080`

### Langkah 2: Jalankan Klien UI (Repositori Tutorial 3)
1. Buka terminal baru dan arahkan ke direktori proyek klien Yew (`tutorial-3-client`).
2. Jalankan *development server* menggunakan **Trunk** pada port `8000` (agar tidak bertabrakan dengan port server backend):
   ```bash
   trunk serve --port 8000 --open
   ```
3. Peramban (*browser*) akan otomatis terbuka pada alamat **`http://localhost:8000`**.
4. Silakan buka beberapa tab baru pada alamat tersebut untuk menguji fitur multi-user chat, daftar online users, dan fungsionalitas pengiriman tombol Enter yang semuanya dikelola di balik layar oleh server Rust!

## 💡 Analisis Eksekusi dan Opini Personal

### 1. *How it was done*
Modifikasi dilakukan dengan mendesain ulang arsitektur penerimaan pesan di sisi server Rust. Alih-alih hanya membaca teks mentah, saya mengimplementasikan pustaka `serde` dan `serde_json` untuk mengubah *string* yang masuk menjadi *struct* Rust (`WebSocketMessage`) secara presisi. Server juga dikonfigurasi untuk menyimpan *state* setiap klien (berupa alamat IP dan *Username* dari *event* `Register`) ke dalam struktur data `HashMap` yang dilindungi oleh `Arc<Mutex>` agar aman diakses secara konkuren oleh ekosistem `tokio`.

### 2. *Why it is a successful change*
Perubahan ini sangat sukses karena server Rust mampu menggantikan server Node.js secara transparan ( *drop-in replacement* ) tanpa memerlukan satu pun perubahan kode di sisi klien (*Yew frontend*). Aplikasi klien tetap berjalan normal; daftar *user online* ter- *update* secara *real-time*, dan pertukaran pesan terjadi tanpa jeda. Ini membuktikan bahwa integrasi pertukaran data JSON lintas bahasa (Rust ke WebAssembly/JavaScript) dapat dieksekusi dengan sempurna melalui protokol WebSocket asalkan *contract data* (struktur JSON) yang disepakati sama.

### 3. Opini: JS vs Rust Preference*
Secara personal, saya lebih menyukai versi **Rust**.
Meskipun versi Node.js (JavaScript) jauh lebih cepat untuk ditulis (*rapid prototyping*) dan kodenya lebih singkat karena tidak perlu mendefinisikan *struct* secara kaku, versi Rust memberikan **ketenangan pikiran (*peace of mind*)**.

Dengan Rust, struktur data JSON dijamin oleh *compiler* melalui `serde`. Jika ada bentuk data yang tidak sesuai (misalnya *client* mengirim atribut yang salah), Rust akan langsung mendeteksinya pada tahap *Compile-Time* (atau menanganinya dengan aman lewat `Result/Option` di *runtime*). Sebaliknya, di Node.js, kesalahan format struktur objek berpotensi besar menyebabkan *runtime error* atau *undefined behavior* yang tiba-tiba membuat server *crash*. Selain itu, penanganan konkurensi dengan `tokio` di Rust terasa lebih kokoh untuk skalabilitas jangka panjang dibandingkan arsitektur *single-thread event-loop* milik Node.js.