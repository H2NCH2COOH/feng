use super::{eval_source, parse_str};

#[test]
fn basic() {
    let code = "\
(assert true)
(assert false)
(assert (not ()))
(assert (atom-eq? a a))
(assert (not (atom-eq? a b)))
";
    eval_source(&parse_str(code).unwrap()).unwrap();
}

#[test]
fn basic_eval() {
    let code = "\
(assert (atom-eq? (eval! a) a))
(assert (atom-eq? (eval a) a))
(define! a b)
(define! b c)
(define! c d)
(assert (atom-eq? (eval! a) (quote! b)))
(assert (atom-eq? (eval a) (quote! c)))


(assert (atom-eq? d (quote! d)))
(define c b)
(assert (atom-eq? d (quote! c)))
";
    eval_source(&parse_str(code).unwrap()).unwrap();
}

#[test]
fn basic_upeval() {
    let code = "\
(define! a 1)
(begin!
    (define! a 2)
    (define! b a)
    (assert (atom-eq? (upeval! a) 1))
    (assert (atom-eq? (upeval b) 1)))
(assert (atom-eq? a (quote! 1)))
";
    eval_source(&parse_str(code).unwrap()).unwrap();
}

#[test]
fn basic_fexpr() {
    let code = "\
(define f! (fexpr! (a b) ((atom-eq? a b))))
(assert (f! 1 1))

(define g! (fexpr! args ((eval (cons (quote! atom-concat) args)))))
(assert (g! 1 2 3) 123)
";
    eval_source(&parse_str(code).unwrap()).unwrap();
}

#[test]
fn recursive_upeval() {
    let code = "\
(define thisctx ())
(define upctx (fexpr! args (
    (cond
        args
            (cons (quote! UP) (upeval (car args)))))))
        else
            (quote! (UP))

(define eval-at (fexpr! (v ctx) (
    (define _build_upeval_chain (fexpr! (v ctx) (
        (define (quote! v) (upeval v))
        (define (quote! ctx) (upeval ctx))
        (cond
            ctx
                (list upeval! (_build_upeval_chain v (cdr ctx)))
            else
                (list upeval! v)))))
    (upeval (_build_upeval_chain (upeval v) (upeval ctx))))))

(define f (fexpr! (ctx dep) (
    (define (quote! ctx) (upeval ctx))
    (define (quote! dep) (upeval dep))
    (define (quote! a) dep)
    (cond
        (atom-eq? dep (quote! aaaaaaaaaaaaa))
            (eval-at (quote! a) ctx)
        else
            (f (upctx ctx) (atom-concat dep (quote! a)))))))

(define! a 1)
(begin!
    (assert (atom-eq? (f (upctx) (quote! a)) 1)))
(assert (atom-eq? (f thisctx (quote! a)) 1))
";
    eval_source(&parse_str(code).unwrap()).unwrap();
}
