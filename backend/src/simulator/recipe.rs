use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeStage {
    pub name: String,
    pub duration_seconds: f64,
    pub target_build_plate_temperature_c: f64,
    pub target_chamber_oxygen_ppm: f64,
    pub target_recoater_position_mm: f64,
    pub tolerance_temperature_c: f64,
    pub tolerance_oxygen_ppm: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub description: String,
    pub stages: Vec<RecipeStage>,
}

impl Recipe {
    pub fn lpbf_layer_demo() -> Self {
        Self {
            id: "lpbf_layer_demo".into(),
            name: "LPBF Layer Monitoring Demo".into(),
            description: "A simulated laser powder bed fusion layer with chamber conditioning, recoater motion, thermal control, and spatter monitoring.".into(),
            stages: vec![
                stage(
                    "Inerting",
                    10.0,
                    25.0,
                    80.0,
                    0.0,
                    4.0,
                    80.0,
                    "Reduce chamber oxygen toward the inert process envelope.",
                ),
                stage(
                    "Preheat",
                    14.0,
                    180.0,
                    60.0,
                    0.0,
                    8.0,
                    40.0,
                    "Raise the simulated build plate to its layer preheat target.",
                ),
                stage(
                    "PowderSpread",
                    10.0,
                    180.0,
                    55.0,
                    100.0,
                    6.0,
                    35.0,
                    "Move the recoater across the plate and monitor its vibration.",
                ),
                stage(
                    "LaserScan",
                    26.0,
                    185.0,
                    50.0,
                    100.0,
                    6.0,
                    30.0,
                    "Simulate laser scanning and high-speed spatter observations.",
                ),
                stage(
                    "LayerInspect",
                    10.0,
                    180.0,
                    55.0,
                    0.0,
                    7.0,
                    35.0,
                    "Aggregate layer quality and review detected ejection events.",
                ),
                stage(
                    "Cooldown",
                    10.0,
                    90.0,
                    120.0,
                    0.0,
                    15.0,
                    80.0,
                    "Lower build-plate temperature after the simulated layer.",
                ),
            ],
        }
    }

    pub fn total_duration(&self) -> f64 {
        self.stages.iter().map(|stage| stage.duration_seconds).sum()
    }
}

#[allow(clippy::too_many_arguments)]
fn stage(
    name: &str,
    duration_seconds: f64,
    target_build_plate_temperature_c: f64,
    target_chamber_oxygen_ppm: f64,
    target_recoater_position_mm: f64,
    tolerance_temperature_c: f64,
    tolerance_oxygen_ppm: f64,
    description: &str,
) -> RecipeStage {
    RecipeStage {
        name: name.into(),
        duration_seconds,
        target_build_plate_temperature_c,
        target_chamber_oxygen_ppm,
        target_recoater_position_mm,
        tolerance_temperature_c,
        tolerance_oxygen_ppm,
        description: description.into(),
    }
}

#[derive(Debug, Clone)]
pub struct RecipeSequencer {
    recipe: Recipe,
    current_stage: usize,
    stage_elapsed: f64,
    total_elapsed: f64,
}

impl RecipeSequencer {
    pub fn new(recipe: Recipe) -> Self {
        Self {
            recipe,
            current_stage: 0,
            stage_elapsed: 0.0,
            total_elapsed: 0.0,
        }
    }

    pub fn update(&mut self, dt: f64) {
        if self.is_complete() {
            return;
        }

        self.stage_elapsed += dt;
        self.total_elapsed += dt;

        while let Some(stage) = self.current() {
            if self.stage_elapsed < stage.duration_seconds {
                break;
            }
            self.stage_elapsed -= stage.duration_seconds;
            self.current_stage += 1;
        }
    }

    pub fn current(&self) -> Option<&RecipeStage> {
        self.recipe.stages.get(self.current_stage)
    }

    pub fn stage_progress(&self) -> f64 {
        self.current()
            .map(|stage| (self.stage_elapsed / stage.duration_seconds).clamp(0.0, 1.0))
            .unwrap_or(1.0)
    }

    pub fn overall_progress(&self) -> f64 {
        (self.total_elapsed / self.recipe.total_duration()).clamp(0.0, 1.0)
    }

    pub fn total_elapsed(&self) -> f64 {
        self.total_elapsed
    }

    pub fn stage_elapsed(&self) -> f64 {
        self.stage_elapsed
    }

    pub fn is_complete(&self) -> bool {
        self.current_stage >= self.recipe.stages.len()
    }
}

#[cfg(test)]
mod tests {
    use super::{Recipe, RecipeSequencer};

    #[test]
    fn sequencer_advances_to_the_next_stage() {
        let recipe = Recipe::lpbf_layer_demo();
        let first_name = recipe.stages[0].name.clone();
        let first_duration = recipe.stages[0].duration_seconds;
        let expected_name = recipe.stages[1].name.clone();
        let mut sequencer = RecipeSequencer::new(recipe);

        assert_eq!(sequencer.current().unwrap().name, first_name);
        sequencer.update(first_duration + 0.1);
        assert_eq!(sequencer.current().unwrap().name, expected_name);
    }
}
