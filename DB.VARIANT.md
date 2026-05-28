# Rust Embedded Database Options

Исследование проведено 2026-05-28 в контексте проекта Ollamer.
Задача: замена `index.json` на локальную однофайловую БД для хранения каталога моделей.

## Сравнительная таблица

| Проект | Звёзды | Тип | Файл | Pure Rust | SQL |
|---|---|---|---|---|---|
| **sled** | ~9k | KV (B-tree) | directory | ✓ | ✗ |
| **redb** | ~4.5k | KV (B+tree, MVCC) | `.redb` | ✓ | ✗ |
| **cozo** | ~4k | relational-graph-vector | `.db` | ✓ | частично |
| **fjall** | ~2k | KV (LSM) | directory | ✓ | ✗ |
| **PoloDB** | ~1.2k | document (MongoDB API) | `.db` | ✓ | ✗ |
| **stoolap** | ~1.1k | embedded SQL (MVCC) | `.db` | ✓ | ✓ |
| **native_db** | ~700 | typed KV (над redb) | `.redb` | ✓ | ✗ |
| **rusqlite** | — | SQL (SQLite bindings) | `.db` | ✗ (C) | ✓ |

## Детали

### redb
- GitHub: https://github.com/cberner/redb
- Поддержка: cberner (индивидуальный разработчик), активен
- MVCC, ACID, zero-copy reads, B+tree
- Одиночный `.redb` файл
- **Рекомендован** для Ollamer: быстрый, надёжный, минимальные зависимости

### native_db
- GitHub: https://github.com/vincent-herlemont/native_db
- Поддержка: vincent-herlemont (один человек), ~700 звёзд — небольшое комьюнити
- Обёртка над redb с derive-макросами, индексы по полям, real-time subscriptions
- Последний коммит: май 2026

### sled
- GitHub: https://github.com/spacejam/sled
- Статус: beta (давно), разработка медленная
- Использует директорию, не одиночный файл

### rusqlite
- Bundled SQLite (C), зрелейший вариант
- Полноценный SQL: JOIN, индексы, сложные запросы
- `rusqlite = { version = "0.31", features = ["bundled"] }`

### stoolap
- Новинка (2025-2026), embedded SQL с MVCC
- GitHub: https://github.com/stoolap/stoolap

### PoloDB
- MongoDB-совместимый API, документная модель
- GitHub: https://github.com/PoloDB/PoloDB

## Вывод для Ollamer

| Сценарий | Выбор |
|---|---|
| Просто заменить JSON, быстрые read/write | **redb** |
| Typed структуры с индексами по полям | **native_db** (над redb) |
| Сложные SQL-запросы, фильтры | **rusqlite** |
| Текущий объём (25 моделей) | JSON достаточен |
