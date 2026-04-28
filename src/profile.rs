use std::path::PathBuf;

use eldenring::{cs::SoloParamRepository, param::PHANTOM_PARAM_ST};
use fromsoftware_shared::{FromStatic, InstanceError};
use rune::Any;
use serde::Deserialize;
use tracing::{error, info};

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct Profile {
    pub script: Vec<ScriptOverride>,
    pub param: Vec<ParamOverride>,
    pub chr_id: Vec<ChrIdOverride>,
    pub player: Option<ChrInsOverride>,
    pub summon: Option<ChrInsOverride>,
    pub interval: Option<u64>,
}

const PARAM_STR: &str = "Param";
const CHR_ID_STR: &str = "Chr_Id";
const PLAYER_STR: &str = "Player";
const SUMMON_STR: &str = "Summon";
fn tracing_error(t: &str, p: &u32) {
    error!(
        "Failed to patch [{}] override because PHANTOM_PARAM_ST with ID '{}' doesn't exist",
        t, p
    )
}

impl Profile {
    pub fn patch(&self) -> Result<(), InstanceError> {
        let solo_param = unsafe { SoloParamRepository::instance() }?;

        self.param.iter().for_each(|param_override| {
            param_override.param_id_list.iter().for_each(|param_id| {
                let param = &param_override.phantom_param;
                match solo_param.get_mut::<eldenring::cs::PhantomParam>(*param_id) {
                    Some(row) => param.patch(row),
                    None => tracing_error(PARAM_STR, param_id),
                }
            })
        });

        info!("Patched params for '{}' param entries", self.chr_id.len());

        self.chr_id.iter().for_each(|chr_id_override| {
            let param = &chr_id_override.phantom_param;
            let param_id = chr_id_override.param_id;
            if param_id != 0 && param_id != u32::MAX {
                match solo_param.get_mut::<eldenring::cs::PhantomParam>(param_id) {
                    Some(row) => param.patch(row),
                    None => tracing_error(CHR_ID_STR, &param_id),
                }
            }
        });

        info!("Patched params for '{}' chr_id entries", self.chr_id.len());

        if let Some(player_override) = &self.player {
            let param = &player_override.phantom_param;
            let param_id = &player_override.param_id;
            match solo_param.get_mut::<eldenring::cs::PhantomParam>(*param_id) {
                Some(row) => param.patch(row),
                None => tracing_error(PLAYER_STR, param_id),
            }
            info!("Patched params for the player entry");
        }

        if let Some(summon_override) = &self.summon {
            let param = &summon_override.phantom_param;
            let param_id = &summon_override.param_id;
            match solo_param.get_mut::<eldenring::cs::PhantomParam>(*param_id) {
                Some(row) => param.patch(row),
                None => tracing_error(SUMMON_STR, param_id),
            }
            info!("Patched params for the summon entry");
        }

        info!("Succesfully applied profile patches");

        Ok(())
    }
}

#[derive(Deserialize, Default, Debug)]
pub struct ScriptOverride {
    pub param_id_list: Vec<u32>,
    pub script_path: PathBuf,
}

#[derive(Deserialize, Default, Debug)]
pub struct ChrInsOverride {
    pub param_id: u32,
    #[serde(default = "default_override_ridden")]
    pub override_ridden: bool,
    #[serde(flatten)]
    pub phantom_param: PhantomParam,
}

#[derive(Deserialize, Default, Debug)]
pub struct ChrIdOverride {
    pub param_id: u32,
    pub chr_id_list: Vec<u32>,
    #[serde(flatten)]
    pub phantom_param: PhantomParam,
}

#[derive(Deserialize, Default, Debug)]
pub struct ParamOverride {
    pub param_id_list: Vec<u32>,
    #[serde(flatten)]
    pub phantom_param: PhantomParam,
}

#[derive(Deserialize, Default, Debug, Any)]
#[repr(C)]
pub struct PhantomParam {
    #[rune(get, set)]
    pub edge_color_a: f32,
    #[rune(get, set)]
    pub front_color_a: f32,
    #[rune(get, set)]
    pub diff_mul_color_a: f32,
    #[rune(get, set)]
    pub spec_mul_color_a: f32,
    #[rune(get, set)]
    pub light_color_a: f32,
    #[rune(get, set)]
    pub edge_color_r: u8,
    #[rune(get, set)]
    pub edge_color_g: u8,
    #[rune(get, set)]
    pub edge_color_b: u8,
    #[rune(get, set)]
    pub front_color_r: u8,
    #[rune(get, set)]
    pub front_color_g: u8,
    #[rune(get, set)]
    pub front_color_b: u8,
    #[rune(get, set)]
    pub diff_mul_color_r: u8,
    #[rune(get, set)]
    pub diff_mul_color_g: u8,
    #[rune(get, set)]
    pub diff_mul_color_b: u8,
    #[rune(get, set)]
    pub spec_mul_color_r: u8,
    #[rune(get, set)]
    pub spec_mul_color_g: u8,
    #[rune(get, set)]
    pub spec_mul_color_b: u8,
    #[rune(get, set)]
    pub light_color_r: u8,
    #[rune(get, set)]
    pub light_color_g: u8,
    #[rune(get, set)]
    pub light_color_b: u8,
    #[serde(default = "default_reserve")]
    reserve: [u8; 1],
    #[rune(get, set)]
    pub alpha: f32,
    #[rune(get, set)]
    pub blend_rate: f32,
    #[rune(get, set)]
    pub blend_type: u8,
    #[rune(get, set)]
    pub is_edge_subtract: u8,
    #[rune(get, set)]
    pub is_front_subtract: u8,
    #[rune(get, set)]
    pub is_no2_pass: u8,
    #[rune(get, set)]
    pub edge_power: f32,
    #[rune(get, set)]
    pub glow_scale: f32,
}

impl PhantomParam {
    pub fn from_row(row: &PHANTOM_PARAM_ST) -> Self {
        Self {
            edge_color_a: row.edge_color_a(),
            front_color_a: row.front_color_a(),
            diff_mul_color_a: row.diff_mul_color_a(),
            spec_mul_color_a: row.spec_mul_color_a(),
            light_color_a: row.light_color_a(),
            edge_color_r: row.edge_color_r(),
            edge_color_g: row.edge_color_g(),
            edge_color_b: row.edge_color_b(),
            front_color_r: row.front_color_r(),
            front_color_g: row.front_color_g(),
            front_color_b: row.front_color_b(),
            diff_mul_color_r: row.diff_mul_color_r(),
            diff_mul_color_g: row.diff_mul_color_g(),
            diff_mul_color_b: row.diff_mul_color_b(),
            spec_mul_color_r: row.spec_mul_color_r(),
            spec_mul_color_g: row.spec_mul_color_g(),
            spec_mul_color_b: row.spec_mul_color_b(),
            light_color_r: row.light_color_r(),
            light_color_g: row.light_color_g(),
            light_color_b: row.light_color_b(),
            reserve: [0],
            alpha: row.alpha(),
            blend_rate: row.blend_rate(),
            blend_type: row.blend_type(),
            is_edge_subtract: row.is_edge_subtract(),
            is_front_subtract: row.is_front_subtract(),
            is_no2_pass: row.is_no2_pass(),
            edge_power: row.edge_power(),
            glow_scale: row.glow_scale(),
        }
    }
    pub fn patch(&self, row: &mut PHANTOM_PARAM_ST) {
        row.set_edge_color_a(self.edge_color_a);
        row.set_front_color_a(self.front_color_a);
        row.set_diff_mul_color_a(self.diff_mul_color_a);
        row.set_spec_mul_color_a(self.spec_mul_color_a);
        row.set_light_color_a(self.light_color_a);
        row.set_edge_color_r(self.edge_color_r);
        row.set_edge_color_g(self.edge_color_g);
        row.set_edge_color_b(self.edge_color_b);
        row.set_front_color_r(self.front_color_r);
        row.set_front_color_g(self.front_color_g);
        row.set_front_color_b(self.front_color_b);
        row.set_diff_mul_color_r(self.diff_mul_color_r);
        row.set_diff_mul_color_g(self.diff_mul_color_g);
        row.set_diff_mul_color_b(self.diff_mul_color_b);
        row.set_spec_mul_color_r(self.spec_mul_color_r);
        row.set_spec_mul_color_g(self.spec_mul_color_g);
        row.set_spec_mul_color_b(self.spec_mul_color_b);
        row.set_light_color_r(self.light_color_r);
        row.set_light_color_g(self.light_color_g);
        row.set_light_color_b(self.light_color_b);
        row.set_alpha(self.alpha);
        row.set_blend_rate(self.blend_rate);
        row.set_blend_type(self.blend_type);
        row.set_is_edge_subtract(self.is_edge_subtract);
        row.set_is_front_subtract(self.is_front_subtract);
        row.set_is_no2_pass(self.is_no2_pass);
        row.set_edge_power(self.edge_power);
        row.set_glow_scale(self.glow_scale);
    }
}

impl From<&mut PHANTOM_PARAM_ST> for PhantomParam {
    fn from(value: &mut PHANTOM_PARAM_ST) -> Self {
        let src = value as *const PHANTOM_PARAM_ST as *const PhantomParam;
        unsafe { std::ptr::read(src) }
    }
}

fn default_override_ridden() -> bool {
    true
}

fn default_reserve() -> [u8; 1] {
    [0u8]
}
