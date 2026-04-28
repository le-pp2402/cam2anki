# Huong dan: Move crawler ra ngoai folder cam

## Muc tieu
- Dua phan dieu phoi crawl (build_jobs, crawl orchestration, progress, worker control) ra khoi domain cam.
- Giu cam la layer domain/adapters cho tu dien Cambridge.

## Nguyen tac phan tach
- cam:
  - Chi giu logic phu thuoc nguon Cambridge:
    - scraper
    - parser
    - model du lieu raw/domain
    - utility lien quan Cambridge URL/HTML
- crawler (module moi):
  - Dieu phoi jobs
  - Concurrency va worker pool
  - Progress tracking
  - Retry policy cap ung dung
- anki:
  - Adapter import vao AnkiConnect
- app/pipeline:
  - Noi cac stage: crawl -> transform -> import

## Cau truc de xuat
- src/crawler/
  - mod.rs
  - job.rs
  - runner.rs
  - progress.rs
  - policy.rs
- src/cam/
  - scraper.rs
  - model.rs
  - util.rs
  - downloader.rs
  - mod.rs
- src/app/
  - pipeline.rs
  - mod.rs

## Mapping chuc nang hien tai sang module moi
- src/cam/crawler.rs hien tai:
  - build_jobs -> src/crawler/job.rs
  - process_job orchestration -> src/crawler/runner.rs
  - progress bar + semaphore -> src/crawler/progress.rs + runner.rs
- main:
  - khong goi truc tiep cam::crawler
  - goi app::pipeline hoac crawler::runner

## Ke hoach migration tung buoc (khong downtime)
1. Tao module src/crawler moi va copy interfaces tu cam/crawler.
2. Chuyen build_jobs sang crawler/job, giu API cu de tranh vo import.
3. Chuyen crawl runner sang crawler/runner, giu hanh vi cu.
4. Chuyen progress state ra crawler/progress.
5. Sua app/main goi module crawler moi.
6. Chay compile + smoke test.
7. Xoa hoac deprecate src/cam/crawler.rs.

## Tieu chuan review sau migration
- cam khong con orchestration logic tong.
- crawler module doc lap, co the tai su dung cho nguon khac (khong chi Cambridge).
- app pipeline ro rang theo stage.
- Khong thay doi output cuoi cung cho nguoi dung (ngoai cai tien do).

## Checklist xac nhan
- Build thanh cong.
- Crawl va import van chay dung voi bo tu mau.
- Tien do stage-based van dung cong thuc units = words * 2.
- Log loi ro rang theo tung stage.
