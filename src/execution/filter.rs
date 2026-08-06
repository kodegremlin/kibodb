use crate::{
    error::Error,
    execution::{
        evaluator::Evaluator,
        executor::{ExecutionContext, Executor},
    },
    relation::{schema::Schema, tuple::Tuple, types::Value},
    sql::ast::Expr,
};

/// A logical executor that filters tuples based on a boolean predicate. It fetches
/// tuples from its child executor, evaluates the provided AST expression against
/// each tuple, and yeilds only those evaluating to `true`.
pub struct FilterExecutor {
    /// The child operator in the Volcano pipeline, like a SeqScan or Join.
    child: Box<dyn Executor>,

    /// The logical predicate to evaluate; the WHERE clause.
    predicate: Expr,

    /// The schema of the upcoming tuples, required by the Evaluator to resolve column
    /// names.
    schema: Schema,
}

impl FilterExecutor {
    /// Initializes a new FilterExecutor.
    pub fn new(child: Box<dyn Executor>, predicate: Expr, schema: Schema) -> Self {
        Self {
            child,
            predicate,
            schema,
        }
    }
}

impl Executor for FilterExecutor {
    fn next(&mut self, ctx: &mut ExecutionContext) -> Result<Option<Tuple>, Error> {
        loop {
            let Some(tuple) = self.child.next(ctx)? else {
                return Ok(None); // pipeline exhausted
            };
            let eval_res = Evaluator::evaluate(&self.predicate, &tuple, &self.schema)?;
            match eval_res {
                // Tuple does not satisfy predicate or is Null, loop to the next one.
                Value::Boolean(false) | Value::Null => continue,
                Value::Boolean(true) => return Ok(Some(tuple)),
                _ => {
                    // The expression evaluated to a non-boolean type. Ex: "WHERE 'hello'";
                    return Err(Error::SyntaxErr(
                        "WHERE clause predicate must evaluate to a boolean".into(),
                    ));
                }
            }
        }
    }
}
