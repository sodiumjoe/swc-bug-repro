use swc_core::{
    common::{
        errors::{ColorConfig, Handler},
        sync::Lrc,
        FileName, SourceMap,
    },
    ecma::ast::Module,
};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

pub fn parse(input: &str) -> Module {
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    let fm = cm.new_source_file(FileName::Custom("test.js".into()), input.into());
    let lexer = Lexer::new(
        Syntax::Typescript(Default::default()),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    parser
        .parse_module()
        .map_err(|e| e.into_diagnostic(&handler).emit())
        .expect("failed to parser module")
}

#[cfg(test)]
mod tests {
    use super::*;
    use swc_core::{
        common::{hygiene::*, *},
        ecma::{ast::*, transforms::base::*, visit::*},
    };

    #[test]
    fn it_works() {
        let input = "
import {foo} from 'bar';

{
  const foo = () => {};
}
";

        #[derive(Debug)]
        struct Tester {
            import: Option<Ident>,
            var: Option<Ident>,
        }

        impl Fold for Tester {
            fn fold_import_named_specifier(
                &mut self,
                n: ImportNamedSpecifier,
            ) -> ImportNamedSpecifier {
                self.import = Some(n.local.clone());
                n
            }
            fn fold_var_decl(&mut self, n: VarDecl) -> VarDecl {
                if let Pat::Ident(ident) = &n.decls[0].name {
                    self.var = Some(ident.id.clone());
                }

                n
            }
        }

        let parsed = parse(input);
        println!("{:#?}", parsed);
        let mut tester = Tester {
            import: None,
            var: None,
        };

        let globals = Globals::new();

        GLOBALS.set(&globals, || {
            Program::Module(parsed)
                .fold_with(&mut resolver(Mark::new(), Mark::new(), true))
                .fold_with(&mut tester);
            println!("{:#?}", tester);
            assert!(tester.import.unwrap().span.ctxt != tester.var.clone().unwrap().span.ctxt);
        });
    }
}
