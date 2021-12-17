(def (to_vec_rec ls:list size:int idx:int)
    (if (= idx size)
        none
        else [(get ls idx) (to_vec_rec ls size (+ idx 1))]
    )
)

(def (to_vec ls:list)
    (to_vec_rec ls (len ls) 0)
)

(def (push vec:list item)
    (if (= (get vec 0) none)
        ([item none])
        else [(get vec 0) (push [(get vec 1)] item)]
    )
)
