(def (iter ls size idx fn)
    (if (= idx size)
        none
        else (get [
            (fn (get ls idx))
            (iter ls size (+ idx 1) fn)
        ] 1)
    )
)

(def (for ls fn)
    (iter ls (len ls) 0 fn)
)

(def (fib n)
    (if (< n 2)
        n
        else (+ (fib (- n 1)) (fib (- n 2)))
    )
)

(for (range 1 41) (lambda (n)
    (println (fib n))
))
