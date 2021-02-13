use ecs::Resource;
use file_manipulation::DirPathBuf;

pub trait SettingsTrait: Resource {
    fn asset_tree(&self) -> &DirPathBuf;
    fn title(&self) -> &str;
    fn dimensions(&self) -> (u32, u32);
    fn clear_color(&self) -> [f32; 4];
    fn vsync(&self) -> bool;
    fn msaa(&self) -> u16;
    fn command_escape(&self) -> char;
    fn command_quote(&self) -> char;
    fn command_punctuation(&self) -> char;
}
