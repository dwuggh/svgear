use std::sync::Arc;

use emacs::{Env, Result, UnibyteString};
use svgear::{
    tokio::{
        self,
        sync::{Mutex, RwLock},
        task::JoinSet,
    },
    PaintParams, PaintType, RenderRequest, SvgManager, Svgear,
};

use crate::raw_value::RawValue;

#[derive(Debug, Clone)]
pub struct CallbackWithArg {
    val: RawValue,
    arg: UnibyteString,
}

impl CallbackWithArg {
    pub fn new(val: RawValue, arg: UnibyteString) -> Self {
        Self { val, arg }
    }

    pub fn resolve(self, env: &Env) -> RawValue {
        let mut cb = self.val;
        cb.replace_env(env);
        cb.call(self.arg);
        cb
    }
}

pub struct AsyncGear {
    pub runtime: tokio::runtime::Runtime,
    pub set: Mutex<JoinSet<Result<CallbackWithArg>>>,
    pub gear: Arc<RwLock<Svgear>>,
}

impl AsyncGear {
    pub fn new(exe_path: String) -> Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()?;
        let gear = Arc::new(RwLock::new(Svgear::new(exe_path)));
        let set = Mutex::new(JoinSet::new());
        Ok(Self { runtime, gear, set })
    }

    pub fn render_svg(
        &mut self,
        val: RawValue,
        content: String,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<()> {
        let id = SvgManager::generate_id(&content);
        let obj = self.gear.clone();
        self.runtime.block_on(async {
            self.set.lock().await.spawn(async move {
                let mut gear = obj.write_owned().await;
                let resp = gear.manager.process_render_request(RenderRequest {
                    id: Some(id),
                    svg_data: content,
                    width,
                    height,
                })?;
                let data_str = UnibyteString::new(resp.bitmap.data);
                let res = CallbackWithArg::new(val, data_str);
                Ok(res)
            })
        });
        Ok(())
    }

    pub fn render_input(
        &self,
        val: RawValue,
        ty: PaintType,
        content: String,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<()> {
        let obj = self.gear.clone();
        self.runtime.block_on(async {
            self.set.lock().await.spawn(async move {
                let mut gear = obj.write_owned().await;
                let svg_data = gear.painter.paint(PaintParams { ty, content }).await?;
                println!("render_input: {svg_data}");
                let id = SvgManager::generate_id(&svg_data);
                let resp = gear.manager.process_render_request(RenderRequest {
                    svg_data,
                    width,
                    height,
                    id: Some(id),
                })?;
                let data_str = UnibyteString::new(resp.bitmap.data);
                let res = CallbackWithArg::new(val, data_str);
                Ok(res)
            })
        });
        Ok(())
    }

    pub async fn join_one(&self, env: &Env) -> Result<()> {
        if let Some(h) = self.set.lock().await.join_next().await {
            let h = h??;
            let cb = h.resolve(env);
            println!("joined one: {cb:?}");
            cb.free_global();
        }
        Ok(())
    }

    pub async fn join_all(&self, env: &Env) -> Result<()> {
        while let Some(h) = self.set.lock().await.join_next().await {
            let h = h??;
            let cb = h.resolve(env);
            println!("joined one: {cb:?}");
            cb.free_global();
        }
        Ok(())
    }

}

