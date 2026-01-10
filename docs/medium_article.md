# Rust ile Modern HTTP İstemcisi ve Performans Testi Aracı Geliştirmek: hurley

Merhaba değerli okuyucular! Bu yazıda sizlere uzun süredir üzerinde çalıştığım ve son derece heyecan duyduğum bir projemi tanıtmak istiyorum. **hurley**, Rust programlama dili ile sıfırdan geliştirdiğim, hem günlük HTTP isteklerinizi kolayca yapmanızı sağlayan hem de API'lerinizin performansını detaylı bir şekilde ölçmenize olanak tanıyan kapsamlı bir komut satırı aracıdır.

---

## Projenin Doğuş Hikayesi ve Motivasyonum

Yazılım geliştirme süreçlerinde API'lerle çalışmak artık kaçınılmaz bir gerçeklik haline geldi. Mikro servis mimarileri, RESTful API'ler ve modern web uygulamaları dünyasında, HTTP istekleri günlük iş akışımızın ayrılmaz bir parçası oldu. Ancak bu istekleri test etmek ve performanslarını ölçmek söz konusu olduğunda, genellikle farklı araçlar arasında geçiş yapmak zorunda kalıyoruz.

Bir yandan basit HTTP istekleri yapmak için bir araç kullanırken, öte yandan yük testleri için bambaşka araçlara başvurmak durumunda kalıyoruz. Bu durum hem iş akışını kesintiye uğratıyor hem de farklı araçların farklı sözdizimlerini öğrenmeyi gerektiriyor. İşte hurley tam da bu sorunu çözmek için tasarlandı.

hurley'in temel felsefesi şudur: **Tek bir araç, iki kritik ihtiyaç**. Günlük HTTP isteklerinizi yaparken aynı araç ile saniyeler içinde performans testine geçebilir, API'nizin yük altındaki davranışını gözlemleyebilirsiniz. Üstelik tüm bunları aşina olduğunuz, tutarlı bir komut satırı arayüzü ile yapabilirsiniz.

---

## Kapsamlı Özellik Seti

hurley, modern bir HTTP istemcisinden beklenen tüm özellikleri sunmanın yanı sıra, profesyonel düzeyde performans testi yetenekleri de barındırmaktadır.

### 🌐 HTTP İstemci Özellikleri

hurley, HTTP protokolünün en yaygın kullanılan tüm metodlarını desteklemektedir. GET, POST, PUT, DELETE, PATCH ve HEAD metodlarının her birini kolayca kullanabilirsiniz. Aşağıda hurley'in sunduğu temel HTTP istemci özelliklerinin detaylı bir listesini bulabilirsiniz:

**Desteklenen HTTP Metodları**: Modern web geliştirmenin gerektirdiği tüm HTTP metodları hurley tarafından tam olarak desteklenmektedir. RESTful API'lerin temel taşı olan CRUD operasyonlarından, daha gelişmiş kullanım senaryolarına kadar her türlü ihtiyacınızı karşılayabilirsiniz.

**Özelleştirilebilir Header Desteği**: API isteklerinizde sıklıkla özel header'lar göndermeniz gerekir. Content-Type, Authorization, Accept ve benzeri standart header'ların yanı sıra, uygulamanıza özgü özel header'ları da kolayca ekleyebilirsiniz. hurley, `-H` parametresi ile sınırsız sayıda header eklemenize olanak tanır.

**Esnek Request Body Seçenekleri**: POST, PUT ve PATCH isteklerinde body göndermeniz gerektiğinde, hurley size iki farklı yöntem sunar. `-d` parametresi ile body içeriğini doğrudan komut satırında tanımlayabilirsiniz. Daha büyük ve karmaşık payload'lar için ise `-f` parametresi ile bir dosyadan body içeriğini okutabilirsiniz.

**Otomatik Redirect Takibi**: Web'de yönlendirmeler son derece yaygındır. hurley, `-L` parametresi ile HTTP 3xx yönlendirmelerini otomatik olarak takip edebilir ve sizi nihai hedefe ulaştırabilir.

**Detaylı Verbose Çıktısı**: Hata ayıklama sırasında isteğin ve yanıtın tüm detaylarını görmek kritik önem taşır. `-v` parametresi ile gönderilen request header'larından, alınan response header'larına kadar her şeyi görüntüleyebilirsiniz.

```bash
# Temel GET isteği örneği
hurley https://api.example.com/users

# JSON formatında veri gönderen POST isteği
hurley -X POST https://api.example.com/users \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-token-here" \
  -d '{"name": "Ahmet Yılmaz", "email": "ahmet@example.com", "role": "developer"}'

# Response header'larını da görüntüleyen istek
hurley -i https://api.example.com/status

# Yönlendirmeleri takip eden istek
hurley -L https://api.example.com/legacy-endpoint

# Tüm detayları gösteren verbose mod
hurley -v https://api.example.com/debug
```

### 🚀 Profesyonel Performans Testi Özellikleri

hurley'in belki de en güçlü ve ayırt edici özelliği, yerleşik performans testi yetenekleridir. Herhangi bir ek araç veya kurulum gerektirmeden, doğrudan komut satırından kapsamlı yük testleri gerçekleştirebilirsiniz.

**Eşzamanlı Bağlantı Yönetimi**: `-c` parametresi ile kaç adet eşzamanlı bağlantı açılacağını belirleyebilirsiniz. Bu sayede gerçek dünya senaryolarını simüle edebilir, API'nizin çoklu kullanıcı yükü altındaki davranışını gözlemleyebilirsiniz.

**Toplam İstek Sayısı Kontrolü**: `-n` parametresi ile test sırasında gönderilecek toplam istek sayısını belirlersiniz. Bu, testin kapsamını ve süresini kontrol etmenizi sağlar.

**Esnek Çıktı Formatları**: Test sonuçlarını terminal üzerinde insan tarafından okunabilir formatta görüntüleyebileceğiniz gibi, `--output json` parametresi ile makine tarafından işlenebilir JSON formatında da alabilirsiniz. Bu özellik, CI/CD pipeline'larına entegrasyon için son derece değerlidir.

```bash
# 10 eşzamanlı bağlantı ile toplam 100 istek gönderme
hurley https://api.example.com/endpoint -c 10 -n 100

# Daha yoğun bir yük testi: 50 eşzamanlı bağlantı, 1000 istek
hurley https://api.example.com/endpoint -c 50 -n 1000

# JSON formatında sonuç alma (otomasyon için ideal)
hurley https://api.example.com/endpoint -c 20 -n 200 --output json

# Dataset dosyası ile çeşitli senaryoları test etme
hurley https://api.example.com --perf test-scenarios.json -c 30 -n 500
```

### 📊 Gelişmiş Dataset Desteği

Gerçek dünya performans testleri, tek bir endpoint'e aynı isteği tekrar tekrar göndermekten çok daha karmaşıktır. Uygulamanız farklı endpoint'lere, farklı HTTP metodları ile, farklı payload'larla istekler alır. hurley'in dataset özelliği, bu karmaşık senaryoları modelleyebilmenizi sağlar.

JSON formatında bir dataset dosyası oluşturarak, test sırasında gönderilecek isteklerin çeşitliliğini tanımlayabilirsiniz. Her istek tanımı, method, path, body ve header bilgilerini içerebilir:

```json
[
  {
    "method": "GET",
    "path": "/api/v1/products",
    "headers": {"Accept": "application/json"}
  },
  {
    "method": "GET",
    "path": "/api/v1/products/42",
    "headers": {"Accept": "application/json"}
  },
  {
    "method": "POST",
    "path": "/api/v1/orders",
    "body": {"product_id": 42, "quantity": 2, "customer_id": 1001},
    "headers": {"Content-Type": "application/json", "Authorization": "Bearer test-token"}
  },
  {
    "method": "PUT",
    "path": "/api/v1/customers/1001",
    "body": {"name": "Güncellenmiş Müşteri Adı", "email": "yeni@email.com"}
  },
  {
    "method": "DELETE",
    "path": "/api/v1/cart/items/15"
  }
]
```

Bu dataset ile hurley, belirtilen istekleri rastgele sırayla seçerek gerçekçi bir trafik paterni oluşturur. Böylece uygulamanızın farklı endpoint'lerdeki performansını tek bir testte değerlendirebilirsiniz.

---

## Detaylı Performans Metrikleri ve Analizi

hurley, performans testi sonuçlarını son derece detaylı ve anlaşılır bir formatta sunar. Her test sonrasında aşağıdaki bilgileri içeren kapsamlı bir rapor alırsınız:

```
═══════════════════════════════════════════════════════════
                    PERFORMANCE RESULTS
═══════════════════════════════════════════════════════════

📊 Request Summary
   Total Requests:      1000
   Successful:          987
   Failed:              13
   Error Rate:          1.30%

⏱️  Timing
   Total Duration:      12456.78 ms
   Requests/sec:        80.28

📈 Latency Distribution
   Min:                 12.34 ms
   Max:                 523.67 ms
   Avg:                 98.23 ms
   p50 (Median):        87.45 ms
   p95:                 234.56 ms
   p99:                 412.89 ms

═══════════════════════════════════════════════════════════
```

### Bu Metrikler Ne Anlama Geliyor?

**Request Summary (İstek Özeti)**: Testin genel başarı durumunu gösterir. Toplam istek sayısı, başarılı istekler, başarısız istekler ve hata oranı bu bölümde yer alır. Yüksek hata oranı, uygulamanızın yük altında sorun yaşadığının bir göstergesi olabilir.

**Timing (Zamanlama)**: Testin toplam süresi ve saniye başına düşen istek sayısı (throughput) bu bölümde raporlanır. Requests/sec değeri, uygulamanızın ne kadar yük kaldırabildiğinin en temel göstergesidir.

**Latency Distribution (Gecikme Dağılımı)**: Bu bölüm, performans analizi için en değerli metrikleri içerir:

- **Min**: En hızlı yanıt süresi
- **Max**: En yavaş yanıt süresi
- **Avg**: Ortalama yanıt süresi (dikkatli yorumlanmalı!)
- **p50 (Median)**: İsteklerin %50'sinin bu sürede veya daha kısa sürede tamamlandığını gösterir
- **p95**: İsteklerin %95'inin bu sürede veya daha kısa sürede tamamlandığını gösterir
- **p99**: İsteklerin %99'unun bu sürede veya daha kısa sürede tamamlandığını gösterir

**Neden Percentile'lar Önemlidir?** Ortalama değer yanıltıcı olabilir. Örneğin, 99 istek 50ms'de tamamlanırken 1 istek 5 saniye sürerse, ortalama düşük görünür ancak kullanıcılarınızın %1'i kötü bir deneyim yaşar. P95 ve P99 değerleri, bu uç durumları yakalamanızı sağlar ve gerçek kullanıcı deneyimini daha iyi yansıtır.

---

## Teknik Mimari ve Tasarım Kararları

hurley, Rust programlama dilinin sunduğu güvenlik garantileri ve performans özellikleri üzerine inşa edilmiştir. Projenin teknik altyapısını oluştururken, Rust ekosisteminin en olgun ve güvenilir crate'lerini tercih ettim.

### Temel Bağımlılıklar ve Kullanım Amaçları

| Crate | Versiyon | Kullanım Amacı |
|-------|----------|----------------|
| `clap` | 4.4 | Komut satırı argümanlarının ayrıştırılması ve doğrulanması |
| `reqwest` | 0.11 | Asenkron HTTP istemci kütüphanesi |
| `tokio` | 1.x | Asenkron runtime ve task yönetimi |
| `hdrhistogram` | 7.5 | Yüksek hassasiyetli latency histogramları ve percentile hesaplamaları |
| `indicatif` | 0.17 | Terminal üzerinde progress bar ve spinner gösterimi |
| `colored` | 2.0 | Renkli ve stilize terminal çıktısı |
| `serde` / `serde_json` | 1.0 | JSON serialization ve deserialization işlemleri |
| `thiserror` | 1.0 | Ergonomik hata tipi tanımlamaları |

### Modüler Kod Yapısı

Projenin kaynak kodu, sorumlulukları net bir şekilde ayrılmış modüller halinde organize edilmiştir:

```
src/
├── main.rs              # Uygulama giriş noktası ve akış kontrolü
├── cli.rs               # Komut satırı argüman tanımları (clap derive)
├── error.rs             # Özel hata tipleri ve Result alias'ı
├── http/
│   ├── mod.rs           # HTTP modülü public API'si
│   ├── client.rs        # HTTP istemci implementasyonu
│   ├── request.rs       # Request builder pattern implementasyonu
│   └── response.rs      # Response işleme ve formatlama
└── perf/
    ├── mod.rs           # Performans modülü public API'si
    ├── runner.rs        # Asenkron performans test runner
    ├── metrics.rs       # Metrik toplama ve hesaplama
    ├── dataset.rs       # Dataset dosyası parsing
    └── report.rs        # Sonuç raporlama ve formatlama
```

### Asenkron Mimari ve Concurrency Modeli

hurley'in performans testi özelliği, Rust'ın `async/await` sözdizimi ve Tokio runtime üzerine inşa edilmiştir. Bu mimari sayesinde:

- Binlerce eşzamanlı HTTP bağlantısını minimum bellek ve CPU kullanımı ile yönetebiliyoruz
- Her istek bağımsız bir async task olarak çalışıyor, birbirlerini bloklamıyor
- Latency metrikleri, lock-free veri yapıları kullanılarak thread-safe bir şekilde toplanıyor
- Progress bar güncellemeleri, ana test akışını kesintiye uğratmadan gerçekleşiyor

---

## Kurulum Rehberi

hurley'i sisteminize kurmanın birkaç farklı yolu bulunmaktadır.

### Cargo ile Kurulum (Önerilen Yöntem)

Rust toolchain'iniz kuruluysa, en kolay yöntem Cargo paket yöneticisini kullanmaktır:

```bash
cargo install hurley
```

Bu komut, hurley'in en son kararlı sürümünü crates.io üzerinden indirecek, derleyecek ve `~/.cargo/bin` dizinine kuracaktır.

### Kaynak Koddan Derleme

Projenin en son geliştirme sürümünü kullanmak veya katkıda bulunmak istiyorsanız, kaynak koddan derleme yapabilirsiniz:

```bash
# Repository'yi klonlayın
git clone https://github.com/dursunkoc/hurley.git

# Proje dizinine geçin
cd hurley

# Release modunda derleyin (optimizasyonlar aktif)
cargo build --release
```

Derleme tamamlandığında, çalıştırılabilir dosya `target/release/hurley` konumunda oluşacaktır. Bu dosyayı PATH'inizdeki bir dizine kopyalayarak her yerden erişilebilir hale getirebilirsiniz.

---

## Yol Haritası ve Gelecek Planları

hurley aktif olarak geliştirilmeye devam etmektedir. Önümüzdeki dönemde eklenmesi planlanan özellikler şunlardır:

- **HTTP/2 ve HTTP/3 Protokol Desteği**: Modern protokollerin sunduğu performans avantajlarından yararlanmak için
- **HAR Dosyası Export**: Test sonuçlarını HTTP Archive formatında dışa aktarma
- **Prometheus Metrikleri**: Monitoring sistemleriyle entegrasyon için native Prometheus endpoint'i
- **TLS Sertifika Seçenekleri**: Özel CA sertifikaları ve sertifika doğrulama bypass seçenekleri
- **Scripting Desteği**: Lua veya JavaScript ile özelleştirilebilir istek mantığı
- **Distributed Test Modu**: Birden fazla makineden koordineli yük testi

---

## Sonuç

hurley, modern yazılım geliştirme süreçlerinin iki temel ihtiyacını tek bir araçta birleştirmeyi hedefleyen bir projedir. HTTP isteklerinizi hızlıca test etmek istediğinizde pratik bir istemci, API'lerinizin performansını ölçmek istediğinizde ise profesyonel düzeyde bir yük testi aracı olarak hizmet vermektedir.

Rust programlama dilinin sunduğu bellek güvenliği, thread güvenliği ve yüksek performans garantileri, hurley'in güvenilir ve verimli bir şekilde çalışmasını sağlamaktadır. Açık kaynak olarak geliştirilen bu projeye katkıda bulunmak, hata bildirmek veya özellik talep etmek için GitHub sayfasını ziyaret edebilirsiniz.

🔗 **GitHub Repository**: [https://github.com/dursunkoc/hurley](https://github.com/dursunkoc/hurley)

📦 **Crates.io Sayfası**: [https://crates.io/crates/hurley](https://crates.io/crates/hurley)

📚 **Dokümantasyon**: [https://docs.rs/hurley](https://docs.rs/hurley)

---

*Bu yazı hakkındaki görüşlerinizi, sorularınızı ve önerilerinizi yorum bölümünde paylaşabilirsiniz. Projeyi beğendiyseniz GitHub'da yıldız vermeyi ve çevrenizle paylaşmayı unutmayın!*

**#Rust #HTTP #PerformanceTest #CommandLineTool #OpenSource #API #LoadTesting**
