;;; Directory Local Variables         -*- no-byte-compile: t; -*-
;;; For more information see (info "(emacs) Directory Variables")

((fundamental-mode . (
		      (tab-width . 4)
		      (indent-tabs-mode . t)
		      (eval . (local-set-key (kbd "TAB")
					     (lambda ()
					       (interactive)
					       (insert-tab))))
		      (eval . (add-hook 'after-save-hook
					(lambda ()
					  (shell-command "cargo fmt"))
					nil t)))))
