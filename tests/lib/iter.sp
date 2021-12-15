(def (iter ls:list size:int idx:int fn:function)
    (if (= idx size)
        none
        else (get [
            (fn (get ls idx))
            (iter ls size (+ idx 1) fn)
        ] 1)
    )
)

(def (for ls:list fn:function)
    (iter ls (len ls) 0 fn)
)
