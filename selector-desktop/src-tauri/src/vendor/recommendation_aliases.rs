pub fn infer_component_type(module_id: &str) -> Option<String> {
    let lowered = module_id.to_lowercase();
    let component = if lowered.contains("pneumatic-gripper") || lowered.contains("gripper") {
        "手指气缸"
    } else if lowered.contains("pneumatic-slide-table") || lowered.contains("slide-table") {
        "滑台气缸"
    } else if lowered.contains("pneumatic-rotary-actuator") || lowered.contains("rotary-actuator") {
        "旋转气缸"
    } else if lowered.contains("timing-belt") {
        "同步轮同步带"
    } else if lowered.contains("v-belt") {
        "V 带"
    } else if lowered.contains("chain") {
        "链条"
    } else if lowered.contains("gear") {
        "齿轮"
    } else if lowered.contains("reducer") {
        "减速机"
    } else if lowered.contains("linear-module") {
        "直线模组"
    } else if lowered.contains("ball-screw") {
        "滚珠丝杠"
    } else if lowered.contains("linear-bearing") {
        "直线轴承"
    } else if lowered.contains("linear-guide") {
        "直线导轨"
    } else if lowered.contains("rolling-bearing") {
        "滚动轴承"
    } else if lowered.contains("coupling") {
        "联轴器"
    } else if lowered.contains("brake-clutch") {
        "制动器/离合器"
    } else if lowered.contains("cylinder") {
        "气缸"
    } else if lowered.contains("vacuum") {
        "真空"
    } else if lowered.contains("valve") || lowered.contains("flow-control") {
        "电磁阀"
    } else if lowered.contains("servo") || lowered.contains("stepper") {
        "伺服/步进电机"
    } else if lowered.contains("motor") {
        "普通电机"
    } else if lowered.contains("indexer") {
        "分割器"
    } else {
        return None;
    };
    Some(component.to_string())
}

pub(super) fn field_aliases(field: &str) -> Vec<String> {
    match field {
        "outputTorque" | "totalTorque" | "loadTorque" | "requiredTorque" | "designTorque"
        | "gripTorque" | "inputTorque" | "momentLoad" | "loadMoment" => vec![
            "outputTorque",
            "totalTorque",
            "loadTorque",
            "ratedTorque",
            "requiredTorque",
            "designTorque",
            "gripTorque",
            "inputTorque",
            "candidateTorque",
            "allowableMoment",
            "momentLoad",
            "loadMoment",
        ],
        "requiredSpeed" | "motorSpeed" | "outputSpeed" | "shaftSpeed" => vec![
            "requiredSpeed",
            "ratedSpeed",
            "speed",
            "outputSpeed",
            "shaftSpeed",
            "maxInputSpeed",
        ],
        "averageSpeed" | "chainSpeed" | "beltSpeed" => vec![
            "averageSpeed",
            "chainSpeed",
            "beltSpeed",
            "linearSpeed",
            "maxSpeed",
        ],
        "power"
        | "ratedPower"
        | "designPower"
        | "outputPower"
        | "peakPower"
        | "requiredPowerPerBelt" => vec![
            "power",
            "ratedPower",
            "designPower",
            "outputPower",
            "peakPower",
            "candidatePowerRating",
            "requiredPowerPerBelt",
        ],
        "load" | "loadMass" | "force" | "thrust" | "beltForce" | "designForce" | "chainPull"
        | "effectivePull" | "outputForce" | "requiredThrust" | "holdingForce" | "forcePerJaw"
        | "suctionForce" | "guideDesignLoad" | "designLoad" | "loadPerSlider"
        | "equivalentLoad" | "tangentialForce" => vec![
            "load",
            "force",
            "thrust",
            "ratedForce",
            "ratedThrust",
            "candidateRatedThrust",
            "outputForce",
            "requiredThrust",
            "holdingForce",
            "forcePerJaw",
            "suctionForce",
            "designLoad",
            "loadPerSlider",
            "equivalentLoad",
            "tangentialForce",
        ],
        "stroke" => vec!["stroke"],
        "bore" | "boreDiameter" | "cupDiameter" | "shaftDiameter" => {
            vec![
                "bore",
                "boreDiameter",
                "cupDiameter",
                "shaftDiameter",
                "diameter",
            ]
        }
        "centerDistance" | "spaceLimit" | "bendRadius" | "installLength" => vec![
            field,
            "centerDistance",
            "spaceLimit",
            "bendRadius",
            "installLength",
        ],
        "vacuumPressure" => vec!["vacuumPressure"],
        "flowRate" | "continuousFlow" => vec!["flowRate", "continuousFlow", "ratedFlow"],
        "dynamicLoad"
        | "dynamicLoadRating"
        | "requiredDynamicLoadRating"
        | "requiredLoadRating"
        | "loadRating" => vec![
            "dynamicLoad",
            "dynamicLoadRating",
            "requiredDynamicLoadRating",
            "requiredLoadRating",
            "loadRating",
        ],
        "staticLoad" | "staticLoadRating" => vec!["staticLoad", "staticLoadRating"],
        "ratedLife" | "travelLife" | "lifeHours" => vec![
            "ratedLife",
            "travelLife",
            "lifeHours",
            "requiredLifeHours",
            "targetTravelLife",
        ],
        "kineticEnergy" => vec![
            "kineticEnergy",
            "allowableKineticEnergy",
            "ratedKineticEnergy",
        ],
        "inertiaRatio" | "resolution" | "torqueMargin" | "forceMargin" | "momentMargin"
        | "thrustMargin" | "energyMargin" | "staticMargin" | "powerMargin" | "loadMargin"
        | "accuracyMargin" => vec![field],
        _ => return vec![field.to_string()],
    }
    .into_iter()
    .map(ToString::to_string)
    .collect()
}
