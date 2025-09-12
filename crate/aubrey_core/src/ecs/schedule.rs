use crate::ecs::system::System;
use crate::ecs::ecs::Ecs;
use std::collections::HashMap;

pub enum Stage {
    PreStartup,
    Startup,
    PostStartup,
    First,
    PreUpdate,
    Update,
    PostUpdate,
    Last,
}

struct ScheduledSystem {
    order: i32,
    label: Option<&'static str>,
    before: Vec<&'static str>,
    after: Vec<&'static str>,
    sys: Box<dyn System>,
}

pub struct Schedules {
    stages: HashMap<&'static str, Vec<ScheduledSystem>>,
    ran_startup: bool,
}

impl Schedules {
    pub fn new() -> Self {
        let mut stages = HashMap::new();
        for key in [
            key(Stage::PreStartup),
            key(Stage::Startup),
            key(Stage::PostStartup),
            key(Stage::First),
            key(Stage::PreUpdate),
            key(Stage::Update),
            key(Stage::PostUpdate),
            key(Stage::Last),
        ] { stages.insert(key, Vec::new()); }
        Self { stages, ran_startup: false }
    }

    pub fn add_system(&mut self, stage: Stage, sys: Box<dyn System>) {
        self.add_system_with_deps(stage, None, &[], &[], 0, sys);
    }

    pub fn add_system_with_order(&mut self, stage: Stage, order: i32, sys: Box<dyn System>) {
        self.add_system_with_deps(stage, None, &[], &[], order, sys);
    }

    pub fn add_system_with_label(&mut self, stage: Stage, label: &'static str, order: i32, sys: Box<dyn System>) {
        self.add_system_with_deps(stage, Some(label), &[], &[], order, sys);
    }

    pub fn add_system_with_deps(
        &mut self,
        stage: Stage,
        label: Option<&'static str>,
        before: &[&'static str],
        after: &[&'static str],
        order: i32,
        sys: Box<dyn System>,
    ) {
        self.stages.get_mut(key(stage)).unwrap().push(ScheduledSystem {
            order,
            label,
            before: before.to_vec(),
            after: after.to_vec(),
            sys,
        });
    }

    pub fn ensure_startup(&mut self, ecs: &mut Ecs) {
        if self.ran_startup { return; }
        self.run_stage(ecs, Stage::PreStartup);
        self.run_stage(ecs, Stage::Startup);
        self.run_stage(ecs, Stage::PostStartup);
        self.ran_startup = true;
    }

    pub fn run_frame(&mut self, ecs: &mut Ecs) {
        self.ensure_startup(ecs);
        for st in [Stage::First, Stage::PreUpdate, Stage::Update, Stage::PostUpdate, Stage::Last] {
            self.run_stage(ecs, st);
        }
    }

    pub fn run_stage(&mut self, ecs: &mut Ecs, stage: Stage) {
        // Set up Commands per stage
        ecs.insert_resource(crate::ecs::ecs::Commands::default());
        if let Some(list) = self.stages.get_mut(key(stage)) {
            // まず order の小さい順に安定ソート
            // 登録順を安定性のため保持
            let n = list.len();
            let mut indices: Vec<usize> = (0..n).collect();
            indices.sort_by_key(|&i| list[i].order);

            // ラベル依存関係に基づくトポロジカル順序
            let mut label_map: HashMap<&'static str, Vec<usize>> = HashMap::new();
            for (pos, &i) in indices.iter().enumerate() {
                if let Some(lbl) = list[i].label {
                    label_map.entry(lbl).or_default().push(i);
                }
                // pos unused but could be used for stability
                let _ = pos;
            }

            let mut indeg = vec![0usize; n];
            let mut edges: Vec<(usize, usize)> = Vec::new();
            for &i in &indices {
                for b in &list[i].before {
                    if let Some(vs) = label_map.get(b) {
                        for &j in vs { edges.push((i, j)); }
                    }
                }
                for a in &list[i].after {
                    if let Some(vs) = label_map.get(a) {
                        for &j in vs { edges.push((j, i)); }
                    }
                }
            }
            for &(_, v) in &edges { indeg[v] += 1; }

            let mut avail: Vec<usize> = indices.iter().copied().filter(|&i| indeg[i] == 0).collect();
            // 小さい order, 次に登録順(=index) の優先で選ぶ
            let pick_next = |pool: &mut Vec<usize>, list: &Vec<ScheduledSystem>| -> Option<usize> {
                if pool.is_empty() { return None; }
                let mut best = 0;
                for k in 1..pool.len() {
                    let a = pool[best];
                    let b = pool[k];
                    let ka = (list[a].order, a);
                    let kb = (list[b].order, b);
                    if kb < ka { best = k; }
                }
                Some(pool.remove(best))
            };

            let mut topo: Vec<usize> = Vec::with_capacity(n);
            while let Some(u) = pick_next(&mut avail, list) {
                topo.push(u);
                for &(s, t) in &edges {
                    if s == u {
                        indeg[t] -= 1;
                        if indeg[t] == 0 { avail.push(t); }
                    }
                }
            }

            let final_order: Vec<usize> = if topo.len() == n {
                topo
            } else {
                // 依存が循環している等。orderのみの順序にフォールバック
                indices
            };

            // 実行
            for i in final_order {
                let s = &mut list[i];
                s.sys.run(ecs);
            }
        }
        // Apply commands (take ownership) and drop resource
        if let Some(mut cmds) = ecs.remove_resource::<crate::ecs::ecs::Commands>() {
            cmds.apply(ecs);
        }
    }
}

fn key(stage: Stage) -> &'static str {
    match stage {
        Stage::PreStartup => "pre_startup",
        Stage::Startup => "startup",
        Stage::PostStartup => "post_startup",
        Stage::First => "first",
        Stage::PreUpdate => "pre_update",
        Stage::Update => "update",
        Stage::PostUpdate => "post_update",
        Stage::Last => "last",
    }
}

