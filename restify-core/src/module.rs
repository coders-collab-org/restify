use crate::{BoxedControllerFn, BoxedModule};

pub trait Module {
  type Context;
  type ControllerContext;
  type ControllerReturn;

  fn details(
    &self,
    ctx: &mut Self::Context,
  ) -> ModuleDetails<Self::Context, Self::ControllerContext, Self::ControllerReturn>;
}

pub struct ModuleDetails<Ctx, ConCtx, ConRet> {
  pub imports: Vec<BoxedModule<Ctx, ConCtx, ConRet>>,
  pub controllers: Vec<BoxedControllerFn<ConCtx, ConRet>>,
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
  let details = module.details(context);
  controllers.extend(details.controllers);

  for imported_module in details.imports {
    configure_module_recursive(&*imported_module, context, controllers);
  }
}
