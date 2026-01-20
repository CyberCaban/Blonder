use std::{env, fs, path::Path};

// Функция для рекурсивного копирования папки
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !src.exists() {
        return Ok(()); // Исходная папка не существует - ничего не делаем
    }

    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            let dest_path = dst.join(entry.file_name());

            // Проверяем, существует ли файл уже
            if dest_path.exists() {
                // Проверяем, нужно ли обновлять файл
                let src_metadata = entry.metadata()?;
                let dst_metadata = fs::metadata(&dest_path)?;

                if src_metadata.modified()? > dst_metadata.modified()? {
                    fs::copy(entry.path(), &dest_path)?;
                    println!("Обновлен: {}", entry.file_name().to_string_lossy());
                }
            } else {
                fs::copy(entry.path(), &dest_path)?;
                println!("Скопирован: {}", entry.file_name().to_string_lossy());
            }
        }
    }

    Ok(())
}

fn main() {
    // Получаем текущую директорию проекта
    let current_dir = env::current_dir().unwrap();

    let out_dir = env::var("OUT_DIR").unwrap();
    // Получаем путь к target/debug или target/release
    let target_dir = Path::new(&out_dir)
        .parent()
        .unwrap() // target/debug/build
        .parent()
        .unwrap() // target/debug
        .parent()
        .unwrap(); // target

    // копируем ассеты
    if let Err(e) = copy_dir_all(&current_dir.join("assets"), &target_dir.join("assets")) {
        panic!("Не удалось скопировать ассеты: {e}");
    }

    let lib_dir = current_dir.join("libs");

    // Проверяем существование папки libs
    if !lib_dir.exists() {
        panic!("Папка 'libs' не найдена в: {current_dir:?}");
    }

    // Проверяем существование файла glfw3.lib
    #[cfg(target_os = "windows")]
    let glfw_lib = lib_dir.join("glfw3.lib");
    #[cfg(target_os = "linux")]
    let glfw_lib = lib_dir.join("libglfw3.a");
    if !glfw_lib.exists() {
        panic!("Файл 'glfw3.lib' не найден в: {lib_dir:?}");
    }

    println!("cargo:rerun-if-changed={}", lib_dir.display());

    // Указываем линкеру путь к библиотекам
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=glfw3");

    // Системные зависимости для Windows
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=opengl32");
        println!("cargo:rustc-link-lib=gdi32");
        println!("cargo:rustc-link-lib=user32");
        println!("cargo:rustc-link-lib=kernel32");
    }

    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=X11");
        println!("cargo:rustc-link-lib=Xrandr");
        println!("cargo:rustc-link-lib=Xinerama");
        println!("cargo:rustc-link-lib=Xcursor");
        println!("cargo:rustc-link-lib=Xi");
        println!("cargo:rustc-link-lib=pthread");
        println!("cargo:rustc-link-lib=dl");
    }

    println!("Build script completed successfully!");
}
