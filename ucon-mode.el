
(setq ucon-highlights
      '(
        ("<'.*'>" . font-lock-string-face)
        ("\"\"\".*\"\"\"" . font-lock-doc-face)
        ("\".*\"" . font-lock-string-face)
        ("+" . font-lock-keyword-face)
        ("[a-zA-Z][a-zA-Z0-9-_\.]*:" . font-lock-type-face)
        ("'[a-zA-Z][a-zA-Z0-9-_\.]*" . font-lock-constant-face)
        ("@[a-zA-Z][a-zA-Z0-9-_\.]*" . font-lock-variable-name-face)
        ("\=[a-zA-Z][a-zA-Z0-9-_\.]*" . font-lock-variable-name-face)
        ("[a-zA-Z][a-zA-Z0-9-_\.]*" . font-lock-keyword-face)
        ))

(define-derived-mode ucon-mode fundamental-mode "UCON"
  "Major mode for Unobtrusive Construct Syntax"
  (setq font-lock-defaults '(ucon-highlights)))

(provide 'ucon-mode)
