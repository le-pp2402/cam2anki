# TODO: Parallel crawl + import Anki

## Muc tieu
- Thiet ke luong xu ly song song cho quy trinh crawl va import vao Anki.
- Tinh tien do theo don vi cong viec, khong theo so tu da hoan tat.
- Vi du: 5 tu => 10 phan tien do.
  - 5 phan cho crawl (moi tu 1 phan)
  - 5 phan cho import (moi tu 1 phan)

## Quy tac tinh tien do
- Tong so phan tien do:
  - total_units = number_of_words * 2
- Tang tien do:
  - +1 unit khi crawl xong 1 tu (thanh cong hoac that bai deu danh dau da xu ly stage crawl)
  - +1 unit khi import xong 1 tu (thanh cong hoac that bai deu danh dau da xu ly stage import)
- Phan tram:
  - progress_percent = completed_units / total_units * 100

## Trang thai cho moi tu
- queued
- crawling
- crawled_ok | crawled_failed
- import_queued (chi co khi crawled_ok)
- importing
- imported_ok | imported_failed

## Thiet ke song song de xuat
- Pipeline 2 stage voi queue noi bo:
  - Stage A: Crawl workers
  - Stage B: Import workers
- Co che:
  - Input words duoc dua vao crawl queue
  - Crawl worker lay word -> crawl -> cap nhat tien do +1
  - Neu crawl thanh cong: day Entry vao import queue
  - Import worker lay Entry -> goi AnkiConnect addNote -> cap nhat tien do +1
- Gioi han concurrency:
  - crawl_concurrency: 3 den 5
  - import_concurrency: 2 den 4
- Backpressure:
  - import queue co gioi han kich thuoc de tranh tang bo nho

## Yeu cau hien thi tien do
- Hien thi tong quan:
  - completed_units / total_units
  - % tien do
- Hien thi theo stage:
  - Crawl: done/total_words
  - Import: done/total_words
- Hien thi ket qua cuoi:
  - crawl_success, crawl_failed
  - import_success, import_failed

## Retry va xu ly loi
- Crawl:
  - Retry 1-2 lan voi loi tam thoi (timeout, network)
- Import:
  - Retry 1 lan neu AnkiConnect loi ket noi
  - Neu loi model field khong hop le: danh dau failed ngay, khong retry
- Luu danh sach tu failed de co the re-run

## Thu tu trien khai
1. Chot data contract giua crawl stage va import stage (Entry toi thieu).
2. Tao ProgressTracker trung tam quan ly unit progress.
3. Tach runner thanh 2 worker pools (crawl/import).
4. Them channels cho 2 stage va co che dong queue dung luc.
5. Them summary va bao cao loi cuoi chuong trinh.
6. Test voi 5 tu mau de xac nhan tien do 10 phan.
7. Test voi 0 tu, 1 tu, 50 tu.

## Tieu chi hoan thanh
- 5 tu dau vao tao ra tong 10 buoc tien do.
- Chuong trinh xu ly song song, khong block toan bo do import cham.
- Tien do khong nhay sai va khong vuot 100%.
- Co bao cao chi tiet success/failed cho tung stage.
