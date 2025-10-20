# Рекомендации по улучшению архитектуры кода проекта Yeth

## Обзор проекта

Yeth - это утилита для построения графов зависимостей между приложениями и вычисления их хешей. Проект написан на Rust и состоит из нескольких модулей:

- `cli.rs` - обработка аргументов командной строки
- `config.rs` - чтение и парсинг конфигурационных файлов
- `graph.rs` - построение графа зависимостей и топологическая сортировка
- `hash.rs` - вычисление хешей директорий и приложений
- `main.rs` - точка входа и координация работы

## Рекомендации по улучшению архитектуры

### 1. Модульная структура и разделение ответственности

#### Текущее состояние
✅ Хорошо: Проект уже разделен на логические модули с четким разделением ответственности.

#### Рекомендации
- **Создать модуль `error.rs`**: Вынести обработку ошибок в отдельный модуль для улучшения читаемости и переиспользования.
- **Создать модуль `output.rs`**: Вынести логику форматирования вывода в отдельный модуль.
- **Создать модуль `version.rs`**: Вынести логику работы с файлами версий в отдельный модуль.

### 2. Улучшение обработки ошибок

#### Текущее состояние
⚠️ Проблема: Используется `anyhow::Result` повсеместно, что делает ошибки менее структурированными.

#### Рекомендации
```rust
// Создать собственные типы ошибок
#[derive(Debug, thiserror::Error)]
pub enum YethError {
    #[error("Application dependency '{0}' for '{1}' not found")]
    DependencyNotFound(String, String),
    
    #[error("Path dependency '{0}' for '{1}' not found")]
    PathDependencyNotFound(PathBuf, String),
    
    #[error("Circular dependency detected")]
    CircularDependency,
    
    #[error("Failed to read config file: {0}")]
    ConfigReadError(#[from] std::io::Error),
    
    #[error("Failed to parse TOML: {0}")]
    TomlParseError(#[from] toml::de::Error),
}
```

### 3. Улучшение структуры конфигурации

#### Текущее состояние
⚠️ Проблема: Структуры конфигурации не используют все возможности Rust.

#### Рекомендации
```rust
// Добавить валидацию конфигурации при загрузке
impl AppConfig {
    pub fn validate(&self) -> Result<(), YethError> {
        // Проверить корректность зависимостей
        // Проверить корректность исключений
        // и т.д.
    }
}

// Использовать builder pattern для сложных конфигураций
impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }
}
```

### 4. Оптимизация производительности

#### Текущее состояние
⚠️ Проблема: Хеширование может быть неэффективным для больших проектов.

#### Рекомендации
- **Параллельное хеширование**: Использовать `rayon` для параллельного хеширования независимых директорий.
- **Кэширование хешей**: Реализовать кэширование хешей файлов для ускорения повторных вычислений.
- **Инкрементное хеширование**: Отслеживать изменения в файлах и пересчитывать только измененные части.

```rust
use rayon::prelude::*;

// Пример параллельного хеширования
pub fn hash_directory_parallel(path: &PathBuf, exclude: &[ExcludePattern]) -> Result<String> {
    let files: Vec<PathBuf> = collect_files(path, exclude)?;
    
    let hashes: Vec<String> = files
        .par_iter()
        .map(|file| hash_file(file))
        .collect::<Result<Vec<_>, _>>()?;
    
    compute_combined_hash(&hashes)
}
```

### 5. Улучшение тестируемости

#### Текущее состояние
❌ Проблема: Отсутствуют тесты в проекте.

#### Рекомендации
- **Добавить модульные тесты**: Для каждого модуля создать comprehensive тесты.
- **Использовать mock-объекты**: Для файловой системы и внешних зависимостей.
- **Добавить интеграционные тесты**: Для проверки всего процесса работы.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_hash_directory_with_exclusions() {
        let temp_dir = TempDir::new().unwrap();
        // Создать тестовую структуру директорий
        // Проверить корректность хеширования с исключениями
    }
    
    #[test]
    fn test_circular_dependency_detection() {
        // Создать конфигурации с циклическими зависимостями
        // Проверить обнаружение циклов
    }
}
```

### 6. Улучшение CLI интерфейса

#### Текущее состояние
✅ Хорошо: Используется `clap` с автоматической генерацией справки.

#### Рекомендации
- **Добавить подкоманды**: Вместо флагов использовать подкоманды для лучшей организации.
- **Улучшить вывод**: Добавить цветной вывод и прогресс-бары для больших проектов.
- **Добавить интерактивный режим**: Для интерактивного выбора приложений.

```rust
#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Calculate hashes for applications
    Hash {
        #[command(flatten)]
        args: HashArgs,
    },
    /// Show dependency graph
    Graph {
        #[command(flatten)]
        args: GraphArgs,
    },
    /// Validate configuration
    Validate {
        #[command(flatten)]
        args: ValidateArgs,
    },
}
```

### 7. Улучшение алгоритма хеширования

#### Текущее состояние
⚠️ Проблема: Используется простой SHA256 без учета метаданных.

#### Рекомендации
- **Учитывать метаданные**: Добавить в хеш информацию о правах доступа, времени модификации и т.д.
- **Использовать более быстрый алгоритм**: Для больших файлов можно рассмотреть использование xxHash или BLAKE3.
- **Добавить детерминированность**: Убедиться, что хеши одинаковы на разных системах.

```rust
// Использование BLAKE3 для более быстрого хеширования
use blake3::Hasher;

pub fn hash_file_blake3(path: &Path) -> Result<String> {
    let mut hasher = Hasher::new();
    let content = fs::read(path)?;
    hasher.update(&content);
    Ok(format!("{}", hasher.finalize()))
}
```

### 8. Улучшение работы с путями

#### Текущее состояние
⚠️ Проблема: Работа с путями может быть улучшена для кросс-платформенности.

#### Рекомендации
- **Использовать `PathBuf` consistently**: Вместо смешивания `Path` и `PathBuf`.
- **Нормализация путей**: Добавить функцию для нормализации путей перед обработкой.
- **Кросс-платформенные исключения**: Учитывать различия в путях для разных ОС.

```rust
pub fn normalize_path(path: &Path) -> PathBuf {
    path.components()
        .filter(|c| !matches!(c, Component::CurDir))
        .fold(PathBuf::new(), |mut path, comp| {
            match comp {
                Component::ParentDir => {
                    path.pop();
                    path
                }
                _ => {
                    path.push(comp);
                    path
                }
            }
        })
}
```

### 9. Добавление логирования

#### Текущее состояние
❌ Проблема: Отсутствует структурированное логирование.

#### Рекомендации
- **Добавить логирование**: Использовать `tracing` или `log` для структурированного логирования.
- **Уровни логирования**: Добавить разные уровни логирования (debug, info, warn, error).
- **Логирование производительности**: Добавить замеры времени для критических операций.

```rust
use tracing::{info, debug, warn, error, instrument};

#[instrument]
pub fn hash_directory(path: &PathBuf, exclude: &[ExcludePattern]) -> Result<String> {
    debug!("Hashing directory: {}", path.display());
    let start = Instant::now();
    
    // ... хеширование ...
    
    let duration = start.elapsed();
    info!("Directory hashed in {:?}", duration);
    Ok(hash)
}
```

### 10. Улучшение документации

#### Текущее состояние
✅ Хорошо: Есть README.md с подробным описанием.

#### Рекомендации
- **Добавить документацию к коду**: Использовать `rustdoc` для документации API.
- **Добавить примеры использования**: Включить примеры в документацию.
- **Создать руководство по разработке**: Добавить CONTRIBUTING.md с инструкциями для разработчиков.

```rust
/// Computes the hash of a directory, excluding specified patterns.
/// 
/// # Arguments
/// 
/// * `path` - Path to the directory to hash
/// * `exclude` - List of patterns to exclude from hashing
/// 
/// # Returns
/// 
/// SHA256 hash of the directory contents as a hex string
/// 
/// # Examples
/// 
/// ```
/// use yeth::hash::hash_directory;
/// use std::path::PathBuf;
/// 
/// let hash = hash_directory(&PathBuf::from("./src"), &[])?;
/// println!("Directory hash: {}", hash);
/// ```
pub fn hash_directory(path: &PathBuf, exclude: &[ExcludePattern]) -> Result<String> {
    // ...
}
```

## Приоритетность рекомендаций

1. **Высокий приоритет**:
   - Добавление тестов
   - Улучшение обработки ошибок
   - Добавление логирования

2. **Средний приоритет**:
   - Оптимизация производительности
   - Улучшение CLI интерфейса
   - Рефакторинг модульной структуры

3. **Низкий приоритет**:
   - Улучшение алгоритма хеширования
   - Улучшение работы с путями
   - Расширение документации

## Заключение

Проект Yeth имеет хорошую базовую архитектуру, но может быть значительно улучшен в области тестирования, обработки ошибок, производительности и документации. Рекомендации выше помогут сделать код более надежным, производительным и поддерживаемым.

## 11. Разделение на библиотеку и бинарный файл

### Текущее состояние
⚠️ Проблема: Весь код находится в одном бинарном крейте, что затрудняет тестирование и использование как библиотеки.

### Рекомендации

#### Структура проекта
```
src/
├── lib.rs              # Публичный API библиотеки
├── main.rs             # Точка входа для CLI
├── cli/                # Модуль CLI
│   ├── mod.rs
│   └── args.rs
├── config/             # Модуль конфигурации
│   ├── mod.rs
│   └── app.rs
├── graph/              # Модуль графа зависимостей
│   ├── mod.rs
│   └── topological.rs
├── hash/               # Модуль хеширования
│   ├── mod.rs
│   └── calculator.rs
└── error.rs            # Модуль ошибок
```

#### Содержимое lib.rs
```rust
//! # Yeth Library
//! 
//! Библиотека для построения графов зависимостей между приложениями и вычисления их хешей.
//! 
//! ## Пример использования
//! 
//! ```rust
//! use yeth::{YethEngine, Config};
//! use std::path::PathBuf;
//! 
//! let config = Config::builder()
//!     .root(PathBuf::from("./my_project"))
//!     .build()?;
//! 
//! let engine = YethEngine::new(config);
//! let hashes = engine.calculate_hashes()?;
//! 
//! for (app, hash) in hashes {
//!     println!("{}: {}", app, hash);
//! }
//! ```

pub mod cli;
pub mod config;
pub mod error;
pub mod graph;
pub mod hash;

use std::collections::HashMap;
use std::path::PathBuf;

use config::{App, discover_apps};
use error::YethError;
use graph::topological_sort;
use hash::{compute_final_hash, hash_directory, hash_path};

/// Конфигурация для движка Yeth
#[derive(Debug, Clone)]
pub struct Config {
    /// Корневая директория для поиска приложений
    pub root: PathBuf,
}

impl Config {
    /// Создает новый билдер конфигурации
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

/// Билдер для конфигурации
#[derive(Default)]
pub struct ConfigBuilder {
    root: Option<PathBuf>,
    short_hash: bool,
    short_hash_length: usize,
}

impl ConfigBuilder {
    pub fn root(mut self, root: PathBuf) -> Self {
        self.root = Some(root);
        self
    }

    pub fn short_hash(mut self, short_hash: bool) -> Self {
        self.short_hash = short_hash;
        self
    }

    pub fn short_hash_length(mut self, length: usize) -> Self {
        self.short_hash_length = length;
        self
    }

    pub fn build(self) -> Result<Config, YethError> {
        Ok(Config {
            root: self.root.unwrap_or_else(|| PathBuf::from(".")),
            short_hash: self.short_hash,
            short_hash_length: self.short_hash_length,
        })
    }
}

/// Основной движок Yeth
pub struct YethEngine {
    config: Config,
}

impl YethEngine {
    /// Создает новый экземпляр движка с указанной конфигурацией
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Возвращает ссылку на конфигурацию
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Обнаруживает все приложения в корневой директории
    pub fn discover_apps(&self) -> Result<HashMap<String, App>, YethError> {
        discover_apps(&self.config.root)
    }

    /// Вычисляет хеши для всех приложений
    pub fn calculate_hashes(&self) -> Result<HashMap<String, String>, YethError> {
        let apps = self.discover_apps()?;

        if apps.is_empty() {
            return Err(YethError::NoApplicationsFound);
        }

        // Топологическая сортировка
        let topo_order = topological_sort(&apps)?;

        // Вычисление хешей
        let mut hashes = HashMap::new();
        for app_name in topo_order {
            let app = apps.get(&app_name).unwrap();
            let own_hash = hash_directory(&app.dir, &app.exclude_patterns)?;

            // Сбор хешей всех зависимостей
            let mut dep_hashes_owned: Vec<String> = Vec::new();

            for dep in &app.dependencies {
                match dep {
                    config::Dependency::App(dep_name) => {
                        let dep_hash: &String = hashes
                            .get(dep_name)
                            .expect("Dependency not processed in correct order");
                        dep_hashes_owned.push(dep_hash.clone());
                    }
                    config::Dependency::Path(path) => {
                        let path_hash = hash_path(path, &app.exclude_patterns)?;
                        dep_hashes_owned.push(path_hash);
                    }
                }
            }

            let dep_hash_refs: Vec<&str> = dep_hashes_owned.iter().map(|s| s.as_str()).collect();
            let final_hash = compute_final_hash(&own_hash, &dep_hash_refs);

            hashes.insert(app_name.clone(), final_hash);
        }

        Ok(hashes)
    }

    /// Вычисляет хеш для конкретного приложения
    pub fn calculate_app_hash(&self, app_name: &str) -> Result<String, YethError> {
        let hashes = self.calculate_hashes()?;
        
        hashes.get(app_name)
            .cloned()
            .ok_or_else(|| YethError::ApplicationNotFound(app_name.to_string()))
    }

    /// Возвращает граф зависимостей в виде строки
    pub fn dependency_graph(&self) -> Result<String, YethError> {
        let apps = self.discover_apps()?;
        Ok(format_dependency_graph(&apps))
    }

    /// Сохраняет хеши в файлы версий
    pub fn write_versions(&self, hashes: &HashMap<String, String>) -> Result<(), YethError> {
        let apps = self.discover_apps()?;
        
        for (app_name, hash) in hashes {
            if let Some(app) = apps.get(app_name) {
                let version_file = app.dir.join("yeth.version");
                let formatted_hash = if self.config.short_hash {
                    hash.chars().take(self.config.short_hash_length).collect()
                } else {
                    hash.clone()
                };
                std::fs::write(&version_file, formatted_hash)?;
            }
        }
        
        Ok(())
    }
}

/// Форматирует граф зависимостей в виде строки
fn format_dependency_graph(apps: &HashMap<String, App>) -> String {
    let mut result = String::new();
    result.push_str("Dependency graph:\n\n");
    
    let mut sorted_apps: Vec<_> = apps.keys().collect();
    sorted_apps.sort();

    for app_name in sorted_apps {
        let app = apps.get(app_name).unwrap();
        result.push_str(&format!("{}\n", app_name));
        
        if app.dependencies.is_empty() {
            result.push_str("  └─ (no dependencies)\n");
        } else {
            for (i, dep) in app.dependencies.iter().enumerate() {
                let prefix = if i == app.dependencies.len() - 1 {
                    "└─"
                } else {
                    "├─"
                };

                match dep {
                    config::Dependency::App(dep_name) => {
                        result.push_str(&format!("  {} {} (app)\n", prefix, dep_name));
                    }
                    config::Dependency::Path(path) => {
                        let path_str = path.display();
                        let kind = if path.is_file() { "file" } else { "dir" };
                        result.push_str(&format!("  {} {} ({})\n", prefix, path_str, kind));
                    }
                }
            }
        }
        result.push('\n');
    }
    
    result
}
```

#### Содержимое main.rs
```rust
use anyhow::Result;
use clap::Parser;
use std::time::Instant;

use yeth::{cli::Cli, Config, YethEngine};

fn main() -> Result<()> {
    let args = Cli::parse();
    let start_time = Instant::now();

    // Создаем конфигурацию из аргументов CLI
    let config = Config::builder()
        .root(args.root)
        .short_hash(args.short_hash)
        .short_hash_length(args.short_hash_length)
        .build()?;

    // Создаем движок Yeth
    let engine = YethEngine::new(config);

    // Если запрошен граф зависимостей
    if args.show_graph {
        let graph = engine.dependency_graph()?;
        print!("{}", graph);
        return Ok(());
    }

    // Вычисляем хеши
    let hashes = engine.calculate_hashes()?;

    // Сохраняем версии если нужно
    if args.write_versions {
        engine.write_versions(&hashes)?;
    }

    // Выводим результаты
    if let Some(app_name) = &args.app {
        // Вывод для конкретного приложения
        let hash = engine.calculate_app_hash(app_name)?;
        let formatted_hash = format_hash(&hash, &engine.config());
        
        if args.hash_only {
            println!("{}", formatted_hash);
        } else {
            println!("{} {}", formatted_hash, app_name);
        }
    } else {
        // Вывод всех приложений
        let mut sorted_apps: Vec<_> = hashes.keys().collect();
        sorted_apps.sort();
        
        for app in sorted_apps {
            let hash = hashes.get(app).unwrap();
            let formatted_hash = format_hash(hash, &engine.config());
            println!("{} {}", formatted_hash, app);
        }
    }

    // Статистика
    if args.verbose {
        let elapsed_time = start_time.elapsed();
        println!();
        println!("Execution time: {:.2?}", elapsed_time);
        println!("Applications processed: {}", hashes.len());
    }

    Ok(())
}

fn format_hash(hash: &str, config: &Config) -> String {
    if config.short_hash {
        hash.chars().take(config.short_hash_length).collect()
    } else {
        hash.to_string()
    }
}
```

#### Обновление Cargo.toml
```toml
[package]
name = "yeth"
version = "0.1.0"
edition = "2021"

[lib]
name = "yeth"
path = "src/lib.rs"

[[bin]]
name = "yeth"
path = "src/main.rs"

[dependencies]
anyhow = "1.0.100"
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0.228", features = ["derive"] }
sha2 = "0.10.9"
toml = "0.9.7"
walkdir = "2.5.0"
thiserror = "1.0"  # Для структурированных ошибок

[dev-dependencies]
tempfile = "3.8"  # Для тестов
```

#### Пример использования как библиотеки
```rust
// В другом проекте
use yeth::{YethEngine, Config};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Создаем конфигурацию
    let config = Config::builder()
        .root(PathBuf::from("./my_monorepo"))
        .short_hash(true)
        .short_hash_length(8)
        .build()?;
    
    // Создаем движок
    let engine = YethEngine::new(config);
    
    // Получаем хеши всех приложений
    let hashes = engine.calculate_hashes()?;
    
    // Проверяем конкретное приложение
    if let Some(hash) = hashes.get("frontend") {
        println!("Frontend hash: {}", hash);
    }
    
    // Получаем граф зависимостей
    let graph = engine.dependency_graph()?;
    println!("{}", graph);
    
    Ok(())
}
```

#### Преимущества такого подхода

1. **Улучшенная тестируемость**: Легко тестировать отдельные компоненты библиотеки
2. **Переиспользование**: Код можно использовать как библиотеку в других проектах
3. **Четкое разделение**: CLI-логика отделена от основной логики
4. **Гибкость**: Разные интерфейсы (CLI, API, etc.) могут использовать одну и ту же библиотеку
5. **Поддержка**: Легче поддерживать и расширять код

#### Тестирование библиотеки
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;
    
    #[test]
    fn test_engine_calculate_hashes() {
        let temp_dir = TempDir::new().unwrap();
        // Создаем тестовую структуру...
        
        let config = Config::builder()
            .root(temp_dir.path().to_path_buf())
            .build()
            .unwrap();
            
        let engine = YethEngine::new(config);
        let hashes = engine.calculate_hashes();
        
        assert!(hashes.is_ok());
    }
}
```

Такое разделение позволит значительно улучшить архитектуру проекта, сделать его более модульным и тестируемым.
