use crate::{BoxedController, BoxedModule};

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
  ) -> Vec<BoxedController<Self::ControllerContext, Self::ControllerReturn>> {
    let _ = ctx;

    vec![]
  }
}

pub(crate) fn resolve_module<Ctx, ConCtx, ConRet>(
  module: &dyn Module<Context = Ctx, ControllerContext = ConCtx, ControllerReturn = ConRet>,
  context: &mut Ctx,
) -> Vec<BoxedController<ConCtx, ConRet>> {
  let mut controllers = vec![];
  configure_module_recursive(module, context, &mut controllers);

  controllers
}

// Recursive function to configure modules and controllers
fn configure_module_recursive<Ctx, ConCtx, ConRet>(
  module: &dyn Module<Context = Ctx, ControllerContext = ConCtx, ControllerReturn = ConRet>,
  context: &mut Ctx,
  controllers: &mut Vec<BoxedController<ConCtx, ConRet>>,
) {
  controllers.extend(module.controllers(context));

  let imported_modules = module.imports(context);

  for imported_module in imported_modules {
    configure_module_recursive(&*imported_module, context, controllers);
  }
}
