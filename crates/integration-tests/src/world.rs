use candid::{decode_one, encode_args, Principal};
use platform::catalog::SubjectInput;
use platform::registry::RegisterArgs;
use sc_types::{
    Answer, AnswerOption, ApiError, ClassificationSubmission, DiscoveryCategory, Protocol,
    Question, SubjectRef, Task,
};
use sha2::{Digest, Sha256};

use crate::pic::{canister_wasm, user, IcpEnv};

pub struct World {
    pub env: IcpEnv,
    pub admin: Principal,
    pub platform: Principal,
    pub aaa: Principal,
    pub owner: Principal,
    pub operator: Principal,
}

fn sample_ref(id: u32) -> SubjectRef {
    SubjectRef {
        subject_id: id,
        field: "ceers".into(),
        ra_deg: 214.9 + (id as f64 * 0.001),
        dec_deg: 52.8 + (id as f64 * 0.001),
        image_url: format!("https://data.example.com/{id}/rgb.png"),
        image_sha256: vec![1; 32],
        dossier_url: format!("https://data.example.com/{id}/dossier.json"),
        dossier_sha256: vec![2; 32],
        data_version: 1,
    }
}

fn sample_protocol() -> Protocol {
    let option = |id: &str| AnswerOption {
        id: id.into(),
        label: id.into(),
        next: None,
    };
    Protocol {
        version: 1,
        questions: vec![Question {
            id: "q1".into(),
            prompt: "Is it smooth?".into(),
            answers: vec![option("smooth"), option("featured")],
        }],
        discovery_categories: vec![DiscoveryCategory {
            id: "lens".into(),
            label: "Gravitational Lens".into(),
            description: "Arcs or rings".into(),
        }],
        guidance_md: "Look closely at the image.".into(),
    }
}

impl World {
    pub fn new(name: &str, subjects: u32) -> Self {
        let env = IcpEnv::new();
        let admin = user(1);
        let payments = user(2);
        let owner = user(10);
        let operator = user(11);
        let platform = env.install_on("platform", admin, 10_000_000_000_000, 0);
        let ok: Result<(), ApiError> =
            env.update(platform, admin, "admin_set_payments_id", payments);
        assert_eq!(ok, Ok(()));
        let wasm = canister_wasm("aaa");
        let hash = Sha256::digest(&wasm).to_vec();
        let world_call = |method: &str, args: Vec<u8>| {
            let bytes = env.pic.update_call(platform, admin, method, args).unwrap();
            let res: Result<(), ApiError> = decode_one(&bytes).unwrap();
            assert_eq!(res, Ok(()), "{method}");
        };
        world_call(
            "admin_upload_wasm",
            encode_args((1u32, wasm, hash)).unwrap(),
        );
        world_call("admin_approve_wasm", encode_args((1u32,)).unwrap());
        let ok: Result<(), ApiError> =
            env.update(platform, admin, "admin_add_protocol", sample_protocol());
        assert_eq!(ok, Ok(()));
        let ok: Result<(), ApiError> =
            env.update(platform, admin, "admin_set_current_protocol", 1u16);
        assert_eq!(ok, Ok(()));
        let batch: Vec<SubjectInput> = (1..=subjects)
            .map(|id| SubjectInput {
                subject: sample_ref(id),
                gold: None,
            })
            .collect();
        let ok: Result<u32, ApiError> = env.update(platform, admin, "admin_add_subjects", batch);
        assert!(ok.is_ok());

        let subnet = env.pic.topology().get_app_subnets()[0];
        let aaa = env.pic.create_canister_on_subnet(Some(admin), None, subnet);
        env.pic.add_cycles(aaa, 30_000_000_000_000);
        env.pic
            .set_controllers(aaa, Some(admin), vec![owner, platform, admin])
            .unwrap();
        let reg = RegisterArgs {
            canister_id: aaa,
            owner,
            name: name.into(),
            avatar_seed: 42,
        };
        let ok: Result<(), ApiError> = env.update(platform, payments, "register_aaa", reg);
        assert_eq!(ok, Ok(()));
        let world = World {
            env,
            admin,
            platform,
            aaa,
            owner,
            operator,
        };
        world.tick(5);
        let bytes = world
            .env
            .pic
            .update_call(
                aaa,
                owner,
                "add_operator",
                encode_args((operator, "bot-1".to_string(), None::<u64>)).unwrap(),
            )
            .unwrap();
        let ok: Result<(), ApiError> = decode_one(&bytes).unwrap();
        assert_eq!(ok, Ok(()));
        world.tick(5);
        world
    }

    pub fn tick(&self, n: usize) {
        for _ in 0..n {
            self.env.pic.tick();
        }
    }

    pub fn get_task(&self) -> Task {
        let res: Result<Task, ApiError> = self.env.update(self.aaa, self.operator, "get_task", ());
        res.expect("operator get_task")
    }

    pub fn submission(&self, task: &Task) -> ClassificationSubmission {
        ClassificationSubmission {
            task_id: task.task_id,
            answers: vec![Answer {
                question_id: "q1".into(),
                answer_id: "smooth".into(),
            }],
            observed_image_sha256: vec![1; 32],
            discovery: None,
            agent_label: None,
            submitted_by: self.operator,
        }
    }

    pub fn await_ingress<R: candid::CandidType + serde::de::DeserializeOwned>(
        &self,
        msg: pocket_ic::common::rest::RawMessageId,
    ) -> R {
        for _ in 0..200 {
            if let Some(res) = self.env.pic.ingress_status(msg.clone()) {
                let bytes = res.unwrap_or_else(|e| panic!("ingress rejected: {e:?}"));
                return decode_one(&bytes).unwrap();
            }
            self.env.pic.tick();
        }
        panic!("ingress did not settle in 200 rounds");
    }
}
