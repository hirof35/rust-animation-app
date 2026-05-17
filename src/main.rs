use egui_macroquad::macroquad; 
use egui_macroquad::macroquad::prelude::*;
use egui_macroquad::macroquad::models::{Mesh, Vertex};
use egui_macroquad::egui;
use rfd::FileDialog;
use std::fs::File;
use std::time::Instant;

// 💡 ウィンドウ設定の型（Conf）も内部のものを使用
fn window_conf() -> egui_macroquad::macroquad::window::Conf {
    egui_macroquad::macroquad::window::Conf {
        window_title: "Sway GIF Tool (Rust)".to_owned(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

// 💡 マクロのパスをフルパスで指定
#[egui_macroquad::macroquad::main(window_conf)]
async fn main() {
    // ---------------------------------------------------------
    // 💡 【追加】eguiの日本語フォント（豆腐対策）設定
    // ---------------------------------------------------------
    egui_macroquad::ui(|egui_ctx| {
        let mut fonts = egui::FontDefinitions::default();
        
        // Windows標準のMSゴシックを読み込む（環境に合わせてパスを変更可能）
        if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\msgothic.ttc") {
            fonts.font_data.insert(
                "japanese".to_owned(),
                egui::FontData::from_owned(font_data),
            );
            
            // 最優先（Proportional）および等幅（Monospace）フォントとして割り当て
            fonts.families.get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "japanese".to_owned());
            fonts.families.get_mut(&egui::FontFamily::Monospace)
                .unwrap()
                .insert(0, "japanese".to_owned());
                
            egui_ctx.set_fonts(fonts);
        }
    });
    // パラメータ
    let mut amplitude = 20.0f32;
    let mut speed = 1.0f32;
    let mut timer = 0.0f32;
    let loop_time = 2.0f32;

    // 画像の状態管理
    let mut texture: Option<Texture2D> = None;
    let mut image_path_str = String::from("画像が選択されていません");

    // 録画の状態管理
    let mut is_recording = false;
    let mut record_start_time = Instant::now();
    let mut recorded_frames: Vec<Vec<u8>> = Vec::new();

    loop {
        clear_background(LIGHTGRAY);
        let delta = get_frame_time();

        // アニメーションタイマーの更新
        timer += delta * speed;
        let t = (timer * std::f32::consts::TAU) / loop_time;
        let offset = t.sin() * amplitude;

        // --- GUI (egui) 描画 ---
        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Sway Settings").show(egui_ctx, |ui| {
                ui.label(&image_path_str);
                
                // 画像アップロードボタン
                if ui.button("📁 画像を読み込む").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("image", &["png", "jpg", "jpeg"])
                        .pick_file() 
                    {
                        image_path_str = path.display().to_string();
                        if let Ok(bytes) = std::fs::read(path) {
                            let tex = Texture2D::from_file_with_format(&bytes, None);
                            texture = Some(tex);
                        }
                    }
                }

                ui.separator();

                // スライダー
                ui.add(egui::Slider::new(&mut amplitude, 0.0..=100.0).text("強さ"));
                ui.add(egui::Slider::new(&mut speed, 0.1..=5.0).text("速さ"));

                ui.separator();

                // 録画ボタン
                if texture.is_some() {
                    let btn_label = if is_recording { "🔴 録画中..." } else { "🎬 GIF保存 (2秒)" };
                    if ui.add_enabled(!is_recording, egui::Button::new(btn_label)).clicked() {
                        is_recording = true;
                        record_start_time = Instant::now();
                        recorded_frames.clear();
                    }
                } else {
                    ui.label("※録画するには画像を読み込んでください");
                }
            });
        });

        // --- 画像の変形描画セクション ---
        // --- 画像の変形描画セクション ---
        if let Some(ref tex) = texture {
            // 1. 描画ターゲットになるエリアの最大サイズを指定 (GUIの右側の残りスペース)
            let max_display_w = screen_width() - 300.0; // 左側のGUIメニュー（約250〜300px）を避ける
            let max_display_h = screen_height() - 50.0;
            
            // 2. アスペクト比を維持したまま、枠内にぴったり収まるサイズ（w, h）を計算
            let img_w = tex.width();
            let img_h = tex.height();
            let scale = (max_display_w / img_w).min(max_display_h / img_h).min(1.0); // 画面より小さい画像は拡大しない場合は最後を .min(1.0) に、常にぴったり合わせるなら削除
            
            let w = img_w * scale;
            let h = img_h * scale;

            // 3. 画面の右側エリアの中央に配置されるように中心座標を計算
            let center_x = 300.0 + (max_display_w / 2.0); // GUIメニューの右側エリアの中央
            let center_y = screen_height() / 2.0;

            // 4. 4頂点の位置を計算 (揺れのアニメーション offset も適用)
            let tl = Vec2::new(center_x - w / 2.0, center_y - h / 2.0);
            let tr = Vec2::new(center_x + w / 2.0, center_y - h / 2.0);
            let bl = Vec2::new(center_x - w / 2.0 - offset, center_y + h / 2.0);
            let br = Vec2::new(center_x + w / 2.0 - offset, center_y + h / 2.0);

            // 頂点データを作成
            let vertices = vec![
                Vertex { position: tl.extend(0.0), uv: vec2(0.0, 0.0), color: WHITE },
                Vertex { position: tr.extend(0.0), uv: vec2(1.0, 0.0), color: WHITE },
                Vertex { position: br.extend(0.0), uv: vec2(1.0, 1.0), color: WHITE },
                Vertex { position: bl.extend(0.0), uv: vec2(0.0, 1.0), color: WHITE },
            ];
            let indices = vec![0, 1, 2, 0, 2, 3];

            // 描画実行
            gl_use_default_material();
            draw_mesh(&Mesh {
                vertices,
                indices,
                texture: Some(tex.clone()),
            });
            // --- 録画処理 ---
            if is_recording {
                let screen_image = get_screen_data();
                recorded_frames.push(screen_image.bytes);

                if record_start_time.elapsed().as_secs_f32() >= loop_time {
                    is_recording = false;
                    
                    let width = screen_width() as u16;
                    let height = screen_height() as u16;
                    let filename = format!("sway_{}.gif", chrono::Local::now().format("%Y%m%d-%H%M%S"));
                    
                    if let Ok(image_file) = File::create(&filename) {
                        let mut encoder = gif::Encoder::new(image_file, width, height, &[]).unwrap();
                        encoder.set_repeat(gif::Repeat::Infinite).unwrap();
                        
                        for frame_bytes in &recorded_frames {
                            let mut frame = gif::Frame::from_rgba(width, height, &mut frame_bytes.clone());
                            frame.delay = 3; 
                            encoder.write_frame(&frame).unwrap();
                        }
                    }
                    println!("GIF saved as: {}", filename);
                }
            }
        }

        egui_macroquad::draw();
        next_frame().await
    }
}