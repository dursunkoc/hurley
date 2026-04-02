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

### 3.2. Veri Seti (Dataset) Tabanlı ve Parametrik Yük Testi

Gerçek dünya trafik desenlerini simüle etmek amacıyla hurley, veri güdümlü (data-driven) test senaryolarını destekler. Kullanıcılar değişken satırları içeren bir veri dosyası (CSV veya JSON) sağlayarak, `{{sutun_adi}}` yer tutucularını içeren istek şablonları oluşturabilir.

Bu yaklaşım, önbellek (cache) mekanizmalarının yanıltıcı etkilerini (cache warming bias) elimine etmek ve sistemin genel kararlılığını farklı ve gerçekçi girdilerle ölçmek için kritiktir.

*   **Veri Dosyaları:** `--data-file` parametresi ile başlık satırı içeren CSV dosyaları veya JSON obje dizileri desteklenir.
*   **Değişken Değiştirme (Substitution):** `{{user_id}}` gibi yer tutucular URL yolunda (path), HTTP header'larında ve istek gövdesinde (body) kullanılabilir.
*   **Satır Döngüsü (Row Cycling):** Yük testi sırasında, istekler veri satırları arasında deterministik olarak ardışık şekilde döner, böylece tüm verilerin kullanıldığından emin olunur.

\`\`\`bash
# Örnek: CSV veri dosyası kullanan parametrik yük testi
hurley -X POST https://api.example.com/users/{{user_id}} \
  -H "Authorization: Bearer {{api_token}}" \
  -d '{"role": "{{role}}"}' \
  --data-file users.csv -c 10 -n 1000
\`\`\`

\`\`\`json
/* Örnek JSON Veri Seti Şeması (CSV'ye alternatif) */
[
  { "user_id": 101, "api_token": "abc...", "role": "admin" },
  { "user_id": 102, "api_token": "def...", "role": "user" }
]
\`\`\`

### 3.3. İş Akışları (Workflow) ve Koşullu Yürütme

hurley, koşullu yanıtlara dayalı olarak çok adımlı yürütme iş akışlarını destekler. `--workflow` parametresine bir JSON dosyası ileterek, bağımlı istekleri dinamik olarak çalıştırabilir ve önceki adımın sonuçlarına göre mantık oluşturabilirsiniz.

```bash
hurley --workflow flow.json https://httpbin.org
```

**`flow.json` formatı:**

```json
{
  "steps": [
    {
      "id": "get_user",
      "request": {
        "method": "GET",
        "path": "/json"
      }
    },
    {
      "id": "conditional_step",
      "condition": "responses.get_user.slideshow.author == \"Yours Truly\"",
      "request": {
        "method": "POST",
        "path": "/post",
        "body": {"message": "Success! The author matched."}
      }
    }
  ]
}
```

Koşul, önceki adımların JSON yanıtlarına karşı değerlendirilir. Değerleri sorgulamak için `responses.<step_id>.<json_path>` sözdizimi kullanılır.

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