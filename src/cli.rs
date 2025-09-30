use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "yeth")]
#[command(about = "Утилита для построения графа зависимостей между приложениями", long_about = None)]
pub struct Cli {
    /// Корневая директория для поиска приложений
    #[arg(short, long, default_value = ".")]
    pub root: PathBuf,

    /// Имя конкретного приложения для вывода хэша (по умолчанию выводятся все)
    #[arg(short, long)]
    pub app: Option<String>,

    /// Показывать только хэш без имени приложения (работает только с --app)
    #[arg(short = 'H', long, requires = "app")]
    pub hash_only: bool,

    /// Не показывать статистику времени выполнения
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Показать граф зависимостей
    #[arg(short = 'g', long)]
    pub show_graph: bool,
}

