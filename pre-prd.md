# capctl — Pre-PRD Document
## AI Coding Agent Profile Manager

**Версия:** 0.1 (Pre-PRD)  
**Дата:** 2025-01-05  
**Статус:** Draft  

---

## 1. Executive Summary

**capctl** — это менеджер профилей для AI coding agents (Claude Code, OpenAI Codex CLI), который позволяет разработчикам:
- Управлять множеством аккаунтов для каждого инструмента
- Запускать несколько изолированных сессий одновременно
- Отслеживать квоты и лимиты по каждому аккаунту

### Ключевая ценность
Разработчик может одновременно работать в 3 окнах Claude Code и 2 окнах Codex, каждое с разным аккаунтом, без конфликтов конфигурации и переменных окружения.

---

## 2. Problem Statement

### Текущая боль разработчиков

1. **Множество аккаунтов** — У разработчика может быть несколько аккаунтов Claude/OpenAI (личный, рабочий, клиентский)

2. **Квоты и лимиты** — Каждый аккаунт имеет временные квоты (5-часовые, суточные, недельные). Когда квота исчерпана на одном аккаунте, нужно переключиться на другой

3. **Конфликт конфигураций** — Claude Code и Codex хранят credentials в глобальных директориях:
   - Claude Code: `~/.claude/`, `~/.claude.json`
   - Codex CLI: `~/.codex/`, `auth.json`, `config.toml`
   
4. **Невозможность параллельной работы** — Нельзя запустить два терминала с разными аккаунтами одного инструмента — они будут использовать одни и те же credentials

5. **Ручное переключение** — Сейчас для смены аккаунта нужно вручную перелогиниваться или манипулировать файлами конфигурации

### Пример сценария
> Иван работает над проектом клиента (аккаунт A) и параллельно над своим pet-project (аккаунт B). Квота на аккаунте A закончилась. Сейчас ему нужно:
> 1. Закрыть Claude Code
> 2. Выйти из аккаунта A
> 3. Войти в аккаунт B
> 4. Открыть Claude Code заново
>
> С capctl: просто открыть новый терминал с `capctl run client-project` и `capctl run personal`

---

## 3. Target Users

### Primary Persona: Professional Developer
- Активно использует AI coding assistants в ежедневной работе
- Имеет 2-5 аккаунтов (личный, рабочий, разные организации)
- Работает над несколькими проектами параллельно
- Технически грамотный, комфортно работает в CLI
- Платформы: macOS, Linux, Windows (WSL)

### Secondary Persona: Team Lead / Tech Lead
- Управляет доступами команды к AI инструментам
- Нужен контроль расхода квот по проектам/аккаунтам

---

## 4. Core Value Proposition

| Без capctl | С capctl |
|------------|----------|
| 1 активный аккаунт на инструмент | Неограниченное количество профилей |
| Ручное переключение через re-login | Мгновенное переключение одной командой |
| Нельзя запустить параллельно | Полная изоляция — запускай сколько угодно |
| Квоты "в голове" | Визуальный мониторинг квот |
| Конфликты конфигов | Изолированные environments |

---

## 5. User Stories

### Epic 1: Profile Management

| ID | User Story | Priority |
|----|------------|----------|
| US-1.1 | Как разработчик, я хочу создать новый профиль для аккаунта, чтобы использовать его отдельно | Must Have |
| US-1.2 | Как разработчик, я хочу видеть список всех моих профилей, чтобы понимать что у меня настроено | Must Have |
| US-1.3 | Как разработчик, я хочу удалить ненужный профиль | Should Have |
| US-1.4 | Как разработчик, я хочу переименовать профиль | Nice to Have |

### Epic 2: Authentication

| ID | User Story | Priority |
|----|------------|----------|
| US-2.1 | Как разработчик, я хочу авторизовать профиль через браузер, чтобы не вводить credentials вручную | Must Have |
| US-2.2 | Как разработчик, я хочу видеть статус авторизации каждого профиля | Must Have |
| US-2.3 | Как разработчик, я хочу переавторизовать профиль если токен истёк | Should Have |

### Epic 3: Isolated Execution

| ID | User Story | Priority |
|----|------------|----------|
| US-3.1 | Как разработчик, я хочу запустить Claude Code с конкретным профилем в изолированном окружении | Must Have |
| US-3.2 | Как разработчик, я хочу запустить несколько инстансов Claude Code с разными профилями одновременно | Must Have |
| US-3.3 | Как разработчик, я хочу запустить Codex CLI с конкретным профилем | Must Have |
| US-3.4 | Как разработчик, я хочу чтобы мои обычные настройки shell (PATH, aliases) сохранялись при запуске через capctl | Must Have |

### Epic 4: Quota Monitoring

| ID | User Story | Priority |
|----|------------|----------|
| US-4.1 | Как разработчик, я хочу видеть текущее потребление квот по каждому профилю | Should Have |
| US-4.2 | Как разработчик, я хочу видеть сводку по всем профилям в одном месте | Should Have |
| US-4.3 | Как разработчик, я хочу получать уведомление когда квота близка к исчерпанию | Nice to Have |

---

## 6. Feature Specification

### 6.1 MVP Features (v0.1)

#### Profile Management
```bash
capctl profile add <name> --tool <claude|codex>   # Создать профиль
capctl profile list                                # Список профилей
capctl profile remove <name>                       # Удалить профиль
capctl profile show <name>                         # Детали профиля
```

#### Authentication
```bash
capctl auth login <profile>    # Авторизовать профиль
capctl auth status <profile>   # Проверить статус авторизации
capctl auth logout <profile>   # Выйти из профиля
```

**Flow авторизации:**
1. Пользователь запускает `capctl auth login work-claude`
2. capctl показывает ссылку для авторизации
3. Пользователь переходит по ссылке, авторизуется в браузере
4. Claude Code / Codex завершает OAuth flow
5. capctl подтверждает успешную авторизацию
6. Токены сохраняются в изолированной директории профиля

#### Isolated Execution
```bash
capctl run <profile> [-- <args>]   # Запустить инструмент с профилем
capctl shell <profile>              # Открыть shell с окружением профиля

# Примеры:
capctl run work-claude
capctl run personal-codex -- --model gpt-4
capctl shell client-project         # Открывает shell, где claude/codex уже настроены
```

#### Quota Display
```bash
capctl status                  # Сводка по всем профилям
capctl status <profile>        # Детали по конкретному профилю
```

**Пример вывода:**
```
┌─────────────────────────────────────────────────────────────────┐
│ capctl status                                                   │
├─────────────────┬────────┬──────────────┬──────────────────────┤
│ Profile         │ Tool   │ Auth Status  │ Quota (5h)           │
├─────────────────┼────────┼──────────────┼──────────────────────┤
│ work-claude     │ claude │ ✓ Active     │ ████████░░ 78%       │
│ personal-claude │ claude │ ✓ Active     │ ██████████ 100%      │
│ client-project  │ codex  │ ✓ Active     │ ██░░░░░░░░ 15%       │
│ experimental    │ codex  │ ✗ Expired    │ —                    │
└─────────────────┴────────┴──────────────┴──────────────────────┘
```

### 6.2 Post-MVP Features (v0.2+)

| Feature | Description | Version |
|---------|-------------|---------|
| TUI Dashboard | Интерактивный терминальный интерфейс (lazygit-style) | v0.2 |
| Quota Alerts | Уведомления при низких квотах | v0.2 |
| Profile Groups | Группировка профилей по проектам | v0.3 |
| Config Sync | Синхронизация профилей между машинами | v0.3 |
| Desktop App | GUI приложение (Tauri) | v1.0 |
| Team Features | Шаринг профилей в команде | v1.0+ |

---

## 7. Technical Architecture

### 7.1 Directory Structure

```
~/.capctl/
├── config.yaml                    # Глобальный конфиг capctl
├── profiles/
│   ├── work-claude/
│   │   ├── meta.yaml              # Метаданные профиля (name, tool, created_at)
│   │   └── claude/                # CLAUDE_CONFIG_DIR для этого профиля
│   │       ├── settings.json
│   │       ├── credentials.json   # OAuth tokens
│   │       └── ...
│   ├── personal-claude/
│   │   ├── meta.yaml
│   │   └── claude/
│   ├── work-codex/
│   │   ├── meta.yaml
│   │   └── codex/                 # CODEX_HOME для этого профиля
│   │       ├── config.toml
│   │       ├── auth.json
│   │       └── ...
│   └── ...
└── cache/
    └── quotas.json                # Кэш данных о квотах
```

### 7.2 Environment Isolation Mechanism

**Ключевой принцип:** Используем переменные окружения для переопределения config directories.

| Tool | ENV Variable | Default | capctl Override |
|------|--------------|---------|-----------------|
| Claude Code | `CLAUDE_CONFIG_DIR` | `~/.claude` | `~/.capctl/profiles/<name>/claude` |
| Codex CLI | `CODEX_HOME` | `~/.codex` | `~/.capctl/profiles/<name>/codex` |

**Пример запуска:**
```bash
# capctl run work-claude внутри делает:
CLAUDE_CONFIG_DIR=~/.capctl/profiles/work-claude/claude claude
```

**Преимущества подхода:**
- Zero overhead (никаких контейнеров, виртуализации)
- Полная совместимость с оригинальными инструментами
- Работает на всех платформах (macOS, Linux, Windows)
- Можно запускать неограниченное количество параллельных сессий

### 7.3 Authentication Flow

```
┌─────────┐     ┌─────────┐     ┌──────────────┐     ┌─────────────┐
│  User   │     │ capctl  │     │ claude/codex │     │   Browser   │
└────┬────┘     └────┬────┘     └──────┬───────┘     └──────┬──────┘
     │               │                 │                    │
     │ capctl auth   │                 │                    │
     │ login work    │                 │                    │
     │──────────────>│                 │                    │
     │               │                 │                    │
     │               │ spawn with      │                    │
     │               │ isolated env    │                    │
     │               │────────────────>│                    │
     │               │                 │                    │
     │               │    auth URL     │                    │
     │               │<────────────────│                    │
     │               │                 │                    │
     │  "Open this   │                 │                    │
     │   link: ..."  │                 │                    │
     │<──────────────│                 │                    │
     │               │                 │                    │
     │               │                 │     OAuth Flow     │
     │───────────────────────────────────────────────────>│
     │               │                 │                    │
     │               │                 │<───────────────────│
     │               │                 │   callback         │
     │               │                 │                    │
     │               │  process exits  │                    │
     │               │  (success)      │                    │
     │               │<────────────────│                    │
     │               │                 │                    │
     │  "✓ Profile   │                 │                    │
     │   authorized" │                 │                    │
     │<──────────────│                 │                    │
     │               │                 │                    │
```

### 7.4 Quota Monitoring

**Подход:** Парсинг вывода `/status` команды

```bash
# Для Claude Code
claude /status  # Выводит информацию о квотах

# Для Codex
codex /status   # Аналогично
```

**Реализация:**
1. `capctl status` запускает соответствующий инструмент в фоне
2. Отправляет команду `/status`
3. Парсит вывод
4. Кэширует результат в `~/.capctl/cache/quotas.json`
5. Отображает в удобном формате

**Fallback:** Если парсинг не работает — показываем "Unknown" вместо квот

### 7.5 Technology Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Language | **Rust** | Кроссплатформенность, один бинарник, скорость |
| CLI Framework | `clap` | Стандарт для Rust CLI |
| TUI (v0.2) | `ratatui` | Мощный TUI фреймворк |
| Config | `serde` + YAML | Простота, человекочитаемость |
| HTTP (для будущих API) | `reqwest` | Async HTTP client |

**Альтернатива:** Go (если команда более знакома)

### 7.6 Platform Support

| Platform | Support Level | Notes |
|----------|---------------|-------|
| macOS (Intel) | Full | Primary development platform |
| macOS (ARM) | Full | Native ARM binary |
| Linux (x64) | Full | |
| Linux (ARM) | Full | For Raspberry Pi, etc. |
| Windows (WSL2) | Full | Recommended for Windows users |
| Windows (Native) | Partial | PowerShell support, some limitations |

---

## 8. User Experience

### 8.1 First Run Experience

```bash
$ capctl

  Welcome to capctl! 🚀
  
  capctl helps you manage multiple AI coding assistant accounts.
  
  Quick start:
    1. Create a profile:    capctl profile add work --tool claude
    2. Authorize it:        capctl auth login work
    3. Start coding:        capctl run work
  
  Run 'capctl help' for all commands.
```

### 8.2 CLI Design Principles

1. **Predictable** — Команды следуют паттерну `capctl <noun> <verb>` или `capctl <action>`
2. **Helpful errors** — Понятные сообщения об ошибках с suggested fixes
3. **Minimal typing** — Короткие alias'ы для частых команд (`capctl r` = `capctl run`)
4. **Non-destructive** — Опасные операции требуют подтверждения
5. **Scriptable** — Поддержка `--json` для автоматизации

### 8.3 Example Session

```bash
# Утро: настраиваем профили
$ capctl profile add work-anthropic --tool claude
✓ Profile 'work-anthropic' created

$ capctl auth login work-anthropic
Opening authorization link...

  → https://claude.ai/oauth/authorize?...
  
  Please open this link in your browser and complete authorization.
  Waiting...

✓ Profile 'work-anthropic' authorized successfully!

$ capctl profile add personal --tool claude
✓ Profile 'personal' created

$ capctl auth login personal
...
✓ Profile 'personal' authorized successfully!

# Работаем над проектом клиента
$ capctl run work-anthropic
Starting Claude Code with profile 'work-anthropic'...
# [Claude Code запускается]

# В другом терминале — личный проект
$ capctl run personal
Starting Claude Code with profile 'personal'...
# [Ещё один Claude Code запускается с другим аккаунтом]

# Проверяем статус
$ capctl status
┌─────────────────┬────────┬──────────────┬──────────────────────┐
│ Profile         │ Tool   │ Status       │ Quota (5h)           │
├─────────────────┼────────┼──────────────┼──────────────────────┤
│ work-anthropic  │ claude │ ✓ Active     │ ████████░░ 78%       │
│ personal        │ claude │ ✓ Active     │ ██████████ 100%      │
└─────────────────┴────────┴──────────────┴──────────────────────┘
```

---

## 9. Success Metrics

### 9.1 Adoption Metrics
| Metric | Target (3 months) | Target (6 months) |
|--------|-------------------|-------------------|
| GitHub Stars | 500 | 2,000 |
| npm/brew installs | 1,000 | 5,000 |
| Active users (weekly) | 200 | 1,000 |

### 9.2 Engagement Metrics
| Metric | Target |
|--------|--------|
| Profiles per user (avg) | 2.5+ |
| Sessions per week (avg) | 10+ |
| Retention (30-day) | 40%+ |

### 9.3 Quality Metrics
| Metric | Target |
|--------|--------|
| CLI response time | < 100ms |
| Auth success rate | > 95% |
| Crash rate | < 0.1% |

---

## 10. Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Claude/Codex меняют формат конфигов | High | Medium | Версионирование совместимости, быстрые hotfixes |
| OAuth flow меняется | High | Low | Абстракция auth layer, fallback на manual token |
| Квоты недоступны для парсинга | Medium | Medium | Graceful degradation — показываем "Unknown" |
| Конкуренты выпускают аналог | Medium | Medium | Focus на UX и community |
| Anthropic/OpenAI блокируют подход | High | Very Low | Мы используем документированные ENV variables |

---

## 11. Competitive Analysis

| Solution | Approach | Limitations |
|----------|----------|-------------|
| Manual re-login | Logout → Login | Медленно, нельзя параллельно |
| Symlinks hack | Переключение symlinks | Не работает параллельно |
| Docker containers | Изоляция через контейнеры | Высокий overhead, сложность |
| Multiple users (OS) | Разные OS users | Неудобно, overkill |
| **capctl** | ENV-based isolation | ✓ Легковесно, ✓ Параллельно |

---

## 12. Roadmap

### Phase 1: MVP (v0.1)
- [ ] Core CLI structure
- [ ] Profile management (add/list/remove)
- [ ] Authentication flow (Claude Code)
- [ ] Authentication flow (Codex)
- [ ] Isolated execution (`capctl run`)
- [ ] Basic status display
- [ ] Cross-platform builds (macOS, Linux)

### Phase 2: Enhanced (v0.2)
- [ ] TUI dashboard (`capctl ui`)
- [ ] Quota monitoring with caching
- [ ] Windows native support
- [ ] Shell completions (bash, zsh, fish)
- [ ] `capctl shell` command

### Phase 3: Polish (v0.3)
- [ ] Profile groups/tags
- [ ] Export/import profiles
- [ ] Homebrew formula
- [ ] npm package (via pkg)
- [ ] Documentation site

### Phase 4: Desktop (v1.0)
- [ ] Tauri desktop app
- [ ] System tray integration
- [ ] Native notifications
- [ ] Auto-update

---

## 13. Design Decisions

### Default Profile Behavior
- **Decision:** Last used profile becomes default
- **Implementation:** Track `last_used` timestamp in `~/.capctl/config.yaml`
- **Usage:** `capctl run` without arguments uses the last used profile

### Auto-generated Profile Names
- **Decision:** When `capctl profile add --tool claude` is called without a name, generate `profile-<timestamp>`
- **Format:** `profile-20250105-171823` (YYYYMMDD-HHMMSS)
- **Rationale:** Unique, sortable, no conflicts

---

## 14. Open Questions

1. **Quota API** — Стоит ли инвестировать в reverse-engineering API для получения квот, или достаточно парсинга /status?

2. **Monetization** — Планируется ли коммерческая версия? Team features? 

3. **Scope expansion** — Поддерживать ли другие инструменты (Cursor, Aider, Continue) в будущем?

---

## 15. Appendix

### A. Claude Code Config Locations
```
~/.claude/                      # Main config directory
~/.claude.json                  # Global settings
~/.claude/settings.json         # User settings
~/.claude/settings.local.json   # Local overrides
~/.config/claude/               # Alternative location (Linux)
```

ENV override: `CLAUDE_CONFIG_DIR`

### B. Codex CLI Config Locations
```
~/.codex/                       # CODEX_HOME
~/.codex/config.toml            # Main config
~/.codex/auth.json              # Credentials (if file storage)
```

ENV override: `CODEX_HOME`

### C. Reference Commands

```bash
# Claude Code
claude                          # Start interactive mode
claude --version               # Version info
claude /status                 # Show status including quotas

# Codex CLI  
codex                          # Start interactive mode
codex --version               # Version info
codex login                   # Authenticate
codex login status            # Check auth status
```

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2025-01-05 | Ivan | Initial draft |
| 0.1.1 | 2025-01-05 | Ivan | Added default profile and naming decisions |

---

*This is a living document. Please update as decisions are made and requirements evolve.*
