fn main() {
    #[cfg(target_os = "windows")]
    {
        let png_path = "assets/img/Suite-install-1.png";
        let ico_path = "assets/img/icon.ico";

        let img = image::open(png_path).expect("Impossible d'ouvrir le PNG");
        let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

        for &size in &[256u32, 48, 32, 16] {
            let resized = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
            let rgba = resized.to_rgba8();
            let icon_image = ico::IconImage::from_rgba_data(size, size, rgba.into_raw());
            icon_dir
                .add_entry(ico::IconDirEntry::encode(&icon_image).expect("Encodage ICO échoué"));
        }

        let mut file = std::fs::File::create(ico_path).expect("Impossible de créer icon.ico");
        icon_dir.write(&mut file).expect("Écriture ICO échouée");

        winresource::WindowsResource::new()
            .set_icon(ico_path)
            .compile()
            .expect("Compilation des ressources Windows échouée");
    }
}
