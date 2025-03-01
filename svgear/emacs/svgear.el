
(defun svgear-callback (data)
  ;; (print data)
  (let ((img (create-image data 'png t)))
    (put-image img 1)))

(add-to-list 'load-path (expand-file-name "./"))
;; (rs-module/load (expand-file-name "svgear-dyn.so"))
(module-load (expand-file-name "svgear-dyn.so"))
;; (svgear-callback (svgear-test1))

(svgear-render-math-to-png 'svgear-callback "adsf" 1 100 100)
(svgear-render-math-to-png 'svgear-callback "a" 2 100 100)
(svgear-resolve-one)
(svgear-resolve)
;; (svgear-test1)

;; (let ((img (create-image (svgear-test1) 'png t)))
;;   (put-image img 1))




;; (let ((img (create-image (with-temp-buffer (insert-file-contents "./1.png") (string-to-unibyte (buffer-string))) 'png t)))
;;   (put-image img 1))



