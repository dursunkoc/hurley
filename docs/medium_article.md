# Rust ile Geliştirilen Yüksek Performanslı HTTP İstemcisi ve Yük Testi Aracı: hurley

Bu makale, Rust programlama dili kullanılarak geliştirilen, hem genel amaçlı bir HTTP istemcisi hem de performans testi aracı olarak işlev gören **hurley** projesinin teknik mimarisini, yeteneklerini ve kullanım senaryolarını ele almaktadır. Yazılım geliştirme süreçlerinde API testlerinin ve performans analizinin tek bir araç üzerinden yönetilmesinin getirdiği verimlilik avantajları incelenecektir.

---

## 1. Giriş ve Motivasyon

Mikro servis mimarilerinin ve dağıtık sistemlerin yaygınlaşmasıyla birlikte, HTTP protokolü üzerinden gerçekleştirilen iletişim, yazılım ekosisteminin can damarı haline gelmiştir. Bu bağlamda, geliştiricilerin iki temel ihtiyacı ortaya çıkmaktadır: (1) API endpoint'lerinin fonksiyonel doğruluğunu test etmek için esnek bir HTTP istemcisi, (2) Sistemlerin yük altındaki davranışını analiz etmek için performans testi araçları.

Genellikle bu iki ihtiyaç için farklı araç setleri (özelleşmiş HTTP istemcileri ve \`wrk\`, \`Apache Benchmark\` gibi yük testi araçları) kullanılmaktadır. **hurley**, bu iki fonksiyonu tek bir komut satırı arayüzünde (CLI) birleştirerek, geliştirme ve test süreçlerindeki bağlam geçişlerini (context switching) minimize etmeyi ve bütünleşik bir test deneyimi sunmayı hedeflemektedir.

---

## 2. Temel Yetenekler ve HTTP İstemcisi Modu

hurley, modern HTTP standartlarına tam uyumluluk gösteren bir istemci moduna sahiptir. RESTful mimarilerin gereksinim duyduğu tüm temel operasyonları desteklemektedir.

### 2.1. Protokol Desteği ve İstek Yapısı

Araç, standart HTTP metodlarının (GET, POST, PUT, DELETE, PATCH, HEAD) tamamını destekler. İsteklerin konfigürasyonu, komut satırı argümanları üzerinden esnek bir şekilde yapılandırılabilir:

*   **Header Yönetimi:** \`-H\` parametresi ile özel HTTP başlıklarının (headers) tanımlanması.
*   **Payload Yönetimi:** \`-d\` parametresi ile satır içi (inline) veri gönderimi veya \`-f\` parametresi ile dosya tabanlı veri akışı.
*   **Yönlendirme (Redirect) Politikaları:** \`-L\` parametresi ile HTTP 3xx serisi yanıtların otomatik takibi.

\`\`\`bash
# Örnek: Özelleştirilmiş header ve payload içeren POST isteği
hurley -X POST https://api.example.com/v1/resource \\
  -H "Content-Type: application/json" \\
  -H "X-Client-ID: system-a" \\
  -d '{"key": "value", "timestamp": 1678900000}'
\`\`\`

---

## 3. Performans Testi ve Yük Simülasyonu

Aracın ayırt edici özelliği, harici bir konfigürasyona ihtiyaç duymadan, mevcut HTTP isteklerini anlık olarak bir yük testine dönüştürebilme yeteneğidir.

### 3.1. Eşzamanlılık (Concurrency) Modeli

hurley, Rust'ın \`Tokio\` asenkron çalışma zamanı (runtime) üzerine inşa edilmiştir. Bu mimari, sistem kaynaklarını (CPU ve Bellek) minimum seviyede kullanarak yüksek sayıda eşzamanlı bağlantının (concurrent connections) yönetilmesine olanak tanır. \`-c\` (concurrency) ve \`-n\` (total requests) parametreleri ile test senaryosunun yoğunluğu belirlenir.

### 3.2. Veri Seti (Dataset) Tabanlı Stokastik Test

Gerçek dünya trafik desenlerini simüle etmek amacıyla, hurley deterministik olmayan test senaryolarını destekler. JSON formatında tanımlanan bir veri seti üzerinden, farklı endpoint'lere, metodlara ve payload'lara sahip istekler rastgele veya sıralı olarak dağıtılabilir. Bu yaklaşım, önbellek (cache) mekanizmalarının yanıltıcı etkilerini (cache warming bias) elimine etmek ve sistemin genel kararlılığını ölçmek için kritiktir.

\`\`\`json
/* Örnek Veri Seti Şeması */
[
  { "method": "GET", "path": "/api/users/101" },
  { "method": "POST", "path": "/api/orders", "body": { "id": 55, "item": "A-1" } }
]
\`\`\`

---

## 4. Performans Metrikleri ve İstatistiksel Analiz

Test sonuçlarının raporlanmasında, ortalama değerlerin ötesine geçilerek istatistiksel dağılım analizleri sunulmaktadır. **Percentile** (yüzdelik dilim) metrikleri, kuyruk gecikmelerinin (tail latency) tespiti için hayati önem taşır.

Raporlanan temel metrikler şunlardır:

*   **Throughput (İşlem Hacmi):** Saniye başına işlenen istek sayısı (RPS).
*   **Latency Distribution (Gecikme Dağılımı):**
    *   **P50 (Medyan):** İsteklerin %50'sinin tamamlanma süresi.
    *   **P95 ve P99:** Sistemin en yavaş %5 ve %1'lik dilimdeki performansı. Bu değerler, SLA (Service Level Agreement) uyumluluğu için kritik göstergelerdir.
    *   **Jitter:** Yanıt sürelerindeki standart sapma ve değişim aralığı.

\`\`\`text
📊 İstatistiksel Özet
   Total Requests:      1000
   Error Rate:          0.00%
   Requests/sec:        450.25

📈 Gecikme Dağılımı (Latency Percentiles)
   p50 (Median):        45.12 ms
   p95:                120.45 ms
   p99:                210.88 ms
\`\`\`

---

## 5. Teknik Mimari

Proje, Rust ekosisteminin performans ve güvenilirlik odaklı kütüphaneleri üzerine kurgulanmıştır:

*   **Asenkron I/O:** \`tokio\` ve \`reqwest\` kütüphaneleri ile non-blocking I/O işlemleri.
*   **İstatistiksel Hesaplama:** \`hdrhistogram\` kütüphanesi ile yüksek hassasiyetli (high dynamic range) histogram analizi.
*   **Hata Yönetimi:** Güçlü tip sistemi (strong type system) ve \`thiserror\` kütüphanesi ile çalışma zamanı hatalarının deterministik yönetimi.

Bu mimari tercihleri, aracın bellek güvenliğinden (memory safety) ödün vermeden C/C++ seviyesinde performans sunmasını sağlamaktadır.

---

## 6. Sonuç

hurley, modern API geliştirme döngüsünde "fonksiyonel test" ve "performans testi" süreçleri arasındaki bariyerleri kaldırmayı hedefleyen bütünleşik bir araçtır. Rust dilinin sunduğu performans avantajlarını, kullanıcı dostu bir arayüz ile birleştirerek, geliştiricilere ve sistem mühendislerine güçlü bir analiz yeteneği sunmaktadır.

Proje açık kaynak kodlu olarak geliştirilmeye devam etmekte olup, dağıtık test yetenekleri ve HTTP/3 desteği gibi özelliklerin yol haritasına eklenmesi planlanmaktadır.

Proje Kaynak Kodları: [https://github.com/dursunkoc/hurley](https://github.com/dursunkoc/hurley)