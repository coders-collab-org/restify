use crate::{BoxedControllerFn, BoxedModule};

pub trait Module {
  type Context;
  type ControllerContext;
  type ControllerReturn;

  fn imports(
    &self,
    ctx: &mut Self::Context,
  ) -> Vec<BoxedModule<Self::Context, Self::ControllerContext, Self::ControllerReturn>> {
    let _ = ctx;

    vec![]
  }

  fn controllers(
    &self,
    ctx: &mut Self::Context,
  ) -> Vec<BoxedControllerFn<Self::ControllerContext, Self::ControllerReturn>> {
    let _ = ctx;

    vec![]
  }
}

#[allow(dead_code)]
pub(crate) fn resolve_module<Ctx, ConCtx, ConRet>(
  module: &dyn Module<Context = Ctx, ControllerContext = ConCtx, ControllerReturn = ConRet>,
  context: &mut Ctx,
) -> Vec<BoxedControllerFn<ConCtx, ConRet>> {
  let mut controllers = vec![];
  configure_module_recursive(module, context, &mut controllers);

  controllers
}

#[allow(dead_code)]
fn configure_module_recursive<Ctx, ConCtx, ConRet>(
  module: &dyn Module<Context = Ctx, ControllerContext = ConCtx, ControllerReturn = ConRet>,
  context: &mut Ctx,
  controllers: &mut Vec<BoxedControllerFn<ConCtx, ConRet>>,
) {
  controllers.extend(module.controllers(context));

  let imported_modules = module.imports(context);

  for imported_module in imported_modules {
    configure_module_recursive(&*imported_module, context, controllers);
  }
}
