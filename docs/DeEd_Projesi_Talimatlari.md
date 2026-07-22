# DeEd Projesi Talimatları

**Bağlam:** DeEd, `budlum-xyz/budlum` reposu üzerine inşa edilen B.U.D. (Broad Universal Database) ekosisteminin içinde bir **alan (domain)**dır — ayrı bir zincir, ayrı bir repo veya ayrı bir yerleşim (settlement) katmanı değildir. Bu doküman, DeEd'in kavramsal tasarımını Budlum'un mevcut kod tabanındaki gerçek primitiflere (crate'ler, RPC uçları, veri yapıları) eşleyerek bir mimari yorum sunar. Kullanıcının verdiği kavramlar (kimlik, itibar, ruh bağı token, iki kollu katılımcı modeli, Seviye A/B veri işleme, kör hakemlik, STG/NFT ödül, küresel ticarileşme, DeArt) burada **referans** olarak alınmış, Budlum'un gerçek mimarisiyle uyumlu hale getirilmiştir.

---

## 0. Temel İlke: "Alan, ayrı yapı değildir"

Budlum zaten bu ilke üzerine kurulu: her biri kendi konsensüsünü koruyan (PoW, PoS, PoA, BFT, ZK) heterojen alanları tek bir **Evrensel Yerleşim Katmanı**nda birleştiriyor; Budlum bu alanların finality kanıtlarını doğrulayıp `GlobalBlockHeader` üzerinde kayıt altına alıyor. DeEd, DeSci ve DeArt bu modele göre **birer yeni konsensüs mekanizması değil**, aynı iskelet (kimlik, itibar, depolama, ödül, köprü) üzerine oturan **kardeş alanlardır**. Bu yüzden aşağıdaki her bölüm "yeni bir zincir kur" değil, "mevcut modülü genişlet / yeni bir RoleId ve RPC namespace ekle" mantığıyla yazılmıştır.

```
PoW alan   PoS alan   PoA alan   ZK/Özel alan   DeEd alanı   DeSci alanı   DeArt alanı
    \          |          |            |             |            |            /
     \_________|__________|____________|_____________|____________|___________/
                                        |
                                        v
                          DomainFinalityAdapter (alana özel kanıt)
                                        |
                                        v
                    ┌───────────────────────────────────────────┐
                    │           BUDLUM YERLEŞİM KATMANI (L1)     │
                    │  GlobalBlockHeader · BridgeState · nonces  │
                    │  ConsensusDomainRegistry · DomainCommit... │
                    └──────────────────┬──────────────────────┬─┘
                                        │                      │
                                        v                      v
                         B.U.D. Depolama Ağı          BudZKVM (budzero/)
                    (ContentManifest / StorageRegistry) (doğrulanabilir yürütme)
```

---

## 1. Budlum'daki mevcut yapı taşları (özet)

| Yol / Bileşen | Rolü | DeEd için önemi |
|---|---|---|
| `src/domain/` | Alan (domain) kaydı, finality adaptörleri | DeEd yeni bir "alan" olarak buraya kaydolur |
| `src/consensus/` | PoW · PoS · PoA motorları | DeEd kendi konsensüsünü icat etmez, mevcut motorlardan birine (veya alan-özel attestation modeline) yaslanır |
| `src/cross_domain/` | Köprü, mesajlaşma, replay koruması | Küresel ticarileşme / likidite için hazır altyapı |
| `src/execution/` | Tx yürütücü + BudZKVM host | Akıllı sözleşme fabrikası burada çalışır |
| `budzero/` | BudZKVM ISA, VM, derleyici, STARK prover | Seviye B kod/algoritma testleri burada yürütülür |
| `src/rpc/` | JSON-RPC (public + operator, iki ayrı dinleyici) | `deed_*` RPC namespace'i aynı desenle eklenir |
| `src/crypto/` | Ed25519, BLS, Dilithium, PKCS#11 | Merkeziyetsiz kimlik ve şifreleme temeli |
| B.U.D. (`ContentId`/`ContentManifest`/`StorageRegistry`) | İçerik adresleme, deal + challenge ekonomisi | Veri seti/makale/IP kaydı ve doğrulama havuzu için doğrudan yeniden kullanılabilir |

**Kritik kısıt (repo'nun kendi kuralı):** B.U.D.'un hiçbir kritik fonksiyonu (deal açma, ücret ödeme, operatör keşfi, erişilebilirlik denetimi, slashing) merkezi bir ekip servisine bağımlı olamaz; whitelist/admin/pause/freeze hook'u yok. **DeEd tasarımı da bu ilkeyi miras almalıdır** — Seviye A/B işleme, ödül dağıtımı ve itibar güncellemeleri izinsiz (permissionless) ve herhangi bir node tarafından sunulabilir şekilde kurgulanmalı, aksi halde projenin kendi egemenlik ilkesiyle çelişir.

---

## 2. Merkeziyetsiz Kimlik ve İtibar Katmanı

| Kavram (kullanıcı tanımı) | Budlum eşleniği |
|---|---|
| Bireyler, kurumlar, cüzdanlar, donanımlar için dijital kimlik | Hesap adresi (`crypto/`'daki Ed25519/BLS/Dilithium anahtar çiftleri) + bu adrese bağlı, B.U.D.'a bir `ContentManifest` olarak yazılmış DID belgesi (içerik-adresli, değiştirilemez) |
| Doğrulanabilir kimlik bilgileri (VC) | O DID'in `ContentId`'sine referans veren, imzalı attestation kayıtları; gizlilik için mevcut PQ-hibrit şifreleme (Dilithium + Ed25519) kullanılır |
| Ruh bağı token (soulbound) | Yürütme katmanında **devredilemez** yeni bir varlık sınıfı (önerilen): tx executor, `soulbound = true` işaretli varlıklarda mint-kaynağı dışına transferi reddeder |
| İtibar takibi | Hesap başına (`ACCT:<addr>` önekinin genişletilmiş hali) tutulan, her doğrulanmış katkı/olayda güncellenen bir itibar sayacı — PoS'taki validator stake/jailed durum takibiyle aynı deseni izler, ancak slashlenebilir değil, yalnızca eklemeli/azaltmalı bir puan

---

## 3. İki Kollu Katılımcı Modeli

Bu, **node operatör personaları** (`config/personas/*`: user-devnet / developer / enterprise-poa) ile **karıştırılmamalı**. Personalar altyapı katmanında "kim node çalıştırıyor" sorusuna cevap verir; DeEd'in iki kolu ise **uygulama/alan katmanında** "kim ne yapabilir" sorusuna cevap veren yeni RoleId'lerdir — B.U.D.'daki `STORAGE_OPERATOR = RoleId(5)` deseninin devamı gibi düşünülebilir:

| Kol | Önerilen RoleId (örnek) | Getirdiği |
|---|---|---|
| AR-GE direktörleri / girişimciler | `INDUSTRY_SPONSOR` (izinsiz kayıt) | Gerçek dünya problemleri, ticari talep tanımları |
| Akademik yaratıcılar / bilimciler | `RESEARCH_CONTRIBUTOR` (izinsiz kayıt) | Yeni bilgi üretimi, teoriden pratiğe geçiş |

Her iki rol de **kayıt sırasında izin gerektirmez** (B.U.D.'daki storage operatörlüğü gibi) — merkezi bir onay mekanizması, §1'deki egemenlik ilkesini ihlal eder.

---

## 4. Akıllı Sözleşme Fabrikası

İki kolun buluştuğu nokta, `src/execution/` (tx yürütücü + BudZKVM host) ve `budzero/` (ISA, VM, derleyici, STARK prover) üzerine kurulacak bir **DeEd Sözleşme Fabrikası**dır:

- Her işbirliği (endüstri problemi × akademik çözüm) için BudZKVM üzerinde çalışan bir program şablonu örneklenir.
- Çıktı (veri seti, araştırma makalesi, fikri mülkiyet) B.U.D.'daki `ContentManifest` deseniyle içerik-adresli olarak kaydedilir.
- **Dürüstlük notu:** Karmaşık kanıtların tam matematiksel sağlamlığı (`VerifyMerkle`, 64-derinlik) şu an Production ortamında kapalı (Z-B 3.5 kapısı beklemede). Bu nedenle fabrikanın ürettiği "doğrulanabilir yürütme" iddiası, bu kapı kapanana kadar **deneysel/test ağı** seviyesinde değerlendirilmelidir — B.U.D.'un kendi depolama katmanının da aynı sınırlamayı açıkça belirttiği gibi.

---

## 5. Veri İşleme Katmanı

### Seviye A — Benzersizlik / Orijinallik Taraması
- Kayıt öncesi (pre-commit) bir attestation adımı: dokümanlar, kod paketleri, veri setleri, DeEd'e özgü bir `deed_registerManifest` çağrısından **önce** taranır.
- Çalıntı/kopya tespit edilirse kayıt reddedilir — repo genelinde zaten uygulanan "fail-closed" felsefesiyle (RPC auth fail-closed, arşiv fail-closed) tutarlı.

### Seviye B — Kod / Algoritma / Matematiksel Model Testi
- Testler BudZKVM (sanal makine) üzerinde izole şekilde çalıştırılır.
- Sonuçlar, B.U.D.'un deal + challenge ekonomisiyle aynı desende bir havuzda toplanır: `bud_storageOpenChallenge` / `bud_storageAnswerChallenge` / `bud_storageGetOutcome` üçlüsünün DeEd'e uyarlanmış hâli (öneri: yeni bir `ConsensusKind::ResearchAttestation`, mevcut `ConsensusKind::StorageAttestation`'ın kardeşi olarak).
- **Dürüstlük notu:** B.U.D.'daki mevcut `RetrievalChallenge` bugün gerçek bir Proof-of-Storage **değildir** (operatör sadece istenen byte aralığını saklayarak testi geçebilir); tam kanıt yine Z-B 3.5 kapısına bağlı. DeEd'in "ResearchAttestation"ı da aynı ara-dönem sınırlamasını miras alacaktır — bu kapı kapanmadan "kriptografik olarak kanıtlanmış özgünlük/doğruluk" iddiası yapılmamalıdır.

---

## 6. Değerlendirme: Kör Hakemlik + AI/LLM Süzgeci + Ödül

| Kavram | Budlum eşleniği |
|---|---|
| Kör hakemlik (anonimlik) | Mevcut BLS finality protokolündeki iki fazlı desenin (Prevote → Precommit → Sertifika) yeniden kullanımı: hakemler önce **gizlenmiş/hash'lenmiş** bir puan "commit" eder, yeterli commit sayısına ulaşınca puanlar "reveal" edilir — böylece kimlik, sürü psikolojisi oluşmadan gizli kalır |
| AI/LLM süzgeci | Commit-reveal döngüsüne paralel çalışan, otomatik puanlama yapan ayrı bir "değerlendirici" sınıfı; nihai puan, insan hakem + otomatik süzgeç ortalaması olarak state'e yazılır |
| STG/NFT ödül dağıtımı | Blok ödülü deseninin (`total_fees + block_reward`, yürütme sırasında hesaba işlenir) katkı-bazlı hâli — B.U.D.'daki "operatör ödül tahakkuku + slashlenen bond defteri" muhasebesiyle aynı mantık |
| İtibar skoru güncellemesi | §2'deki itibar sayacının, her doğrulanmış olayda ödül krediyle **aynı atomik işlemde** güncellenmesi (domain commitment'lar için zaten kullanılan atomik kalıcılık deseni) |

---

## 7. Küresel Ticarileşme, Varlık Koruması, Likidite

- `src/cross_domain/` zaten "kilitle → bas → yak → kilidi aç" (lock → mint → burn → unlock) akışını kanıt kapılarıyla sağlıyor. DeEd/DeSci varlıkları (IP token'ı, veri seti NFT'si) **yeni bir köprü yazılmadan**, bu mevcut yaşam döngüsü üzerinden diğer alanlara (DeFi likidite havuzları dahil) taşınabilir.
- Varlık koruması, alan düzeyinde zaten var olan **equivocation tespiti + dondurma + bond slashing** mekanizmasından miras alınır: bir alan kötü niyetli/çelişkili commit yaparsa anında donar ve bond'u kesilir. Bu güvenlik modeli DeEd varlıkları için ek kod gerektirmeden geçerlidir.

---

## 8. DeArt ve Diğer Genişlemeler

DeArt, yeni bir zincir ya da yeni bir yerleşim mekanizması **değildir**. Aynı kimlik/itibar/depolama/ödül/köprü iskeleti üzerine oturan, yalnızca `ContentManifest` içerik şeması farklılaşan bir başka RoleId + persona setidir (veri seti/makale yerine medya/sanat eseri). Bu, dokümanın başındaki "alan, ayrı yapı değildir" ilkesinin somut kanıtıdır: DeEd, DeSci ve DeArt aynı temel modülü üç farklı içerik türüyle örnekler.

---

## 9. Öğrenci Kontrollü Paylaşım ve Eğitimci Puanlama

- `ContentManifest`'e önerilen bir **görünürlük alanı** (`public` / `educator-only` / `peer-only`) eklenerek öğrencinin çalışmasını herkese açık oylamaya, seçili eğitimcilere veya yalnızca öğrencilere açması sağlanır.
- Oylama: her `ContentId` için imzalı, hafif bir tx; sonuçlar B.U.D.'un okuma-RPC deseniyle (`bud_storageGetOutcome` benzeri, örn. `deed_getVotes`) sorgulanabilir hale getirilir.
- Eğitimci puanlama: §2'deki itibar modülünün "eğitimci" rolüne uygulanmış hâli; herkese açık, hız-sınırlı, API-anahtarlı public JSON-RPC okuma uçlarıyla (B.U.D. okumalarıyla aynı güvenlik deseni) sorgulanabilir — öğrenciler böylece kendine uygun eğitimciyi seçebilir veya sorunlu geçmişi olan eğitimcileri önceden görebilir.

---

## 10. Dürüstlük Notları / Mevcut Sınırlar

Bu bölüm, repo'nun kendi durum beyanına dayanır ve DeEd tasarımı için de geçerlidir:

- Budlum şu an **v0.3-dev, kontrollü genel-devnet adayı**; bağımsız harici denetimden geçmemiştir; gerçek-değer üretim trafiği için kullanılmamalıdır.
- Z-B Merkle 64-derinlik sağlamlığı Production'da kapalı (Z-B 3.5 kapısı) — bu kapıya dayanan her "doğrulanabilir AR-GE testi" veya "kriptografik özgünlük kanıtı" iddiası, kapı kapanana kadar ara-dönem (interim) olarak işaretlenmelidir.
- Veri egemenliği kuralı: B.U.D.'un kritik fonksiyonlarında whitelist/admin/pause/freeze hook'u yok. DeEd'in Seviye A/B işleme hatları, ödül dağıtımı ve itibar güncellemeleri de **izinsiz ve herhangi bir node tarafından sunulabilir** şekilde tasarlanmalı; aksi halde projenin kendi ilkesiyle çelişir.

---

## 11. Önerilen Uygulama Adımları (taslak)

1. `src/domain/deed/` (yeni modül önerisi): manifest şeması, RoleId tanımları, attestation türleri.
2. Yeni `ConsensusKind::ResearchAttestation` varyantı — mevcut `ConsensusKind::StorageAttestation`'dan türetilir.
3. Yeni RPC namespace: `deed_*` — mevcut `bud_*` ile aynı iki-dinleyicili (public + operator) güvenlik deseni.
4. `budzero/` üzerinde DeEd'e özgü ISA şablonları (sözleşme fabrikası yürütücüsü).
5. `config/personas/` altında değişiklik gerekmez — personalar node-operasyonel katmandır; DeEd rolleri state-machine seviyesinde ayrı bir RoleId katmanıdır.
6. İlerleyen turlarda: soulbound varlık sınıfı için `execution/` içinde transfer-kısıtlama mantığı, `ContentManifest` için görünürlük alanı, itibar sayacı için `ACCT:<addr>` şema genişletmesi.

---

*Bu doküman, `github.com/budlum-xyz/budlum` reposunun 22 Temmuz 2026 tarihli genel README/SPECIFICATION/PERSONAS içeriğine dayanarak hazırlanmış bir mimari yorumdur; repo hızla geliştiği için modül/RPC adları uygulamaya geçmeden önce güncel kod tabanıyla teyit edilmelidir.*
