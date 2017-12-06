
(assert (file (name "conmacro_boot.clp") (is 
	;; Utilities
	(assert (clips:function (name ignore)
	                        (arguments "?v")
				(body "nil")))
	(assert (clips:function (name str-endswithp)
	                        (arguments "?str" "?ending")
				(body "(eq (sub-string (+ 1 (- (str-length ?str) (str-length ?ending))) (str-length ?str) ?str) ?ending))")))
	;; Standard to-verbatim implementations
	(assert (clips:generic (name to-verbatim)))

        (assert (clips:method (name to-verbatim)
	                      (arguments "(?s SYMBOL (eq ?s nil))" "?type")
			      (body "\"\"")))
        (assert (clips:method (name to-verbatim)
	                      (arguments "(?s STRING)" "?type")
			      (body "?s")))
	(assert (clips:method (name to-verbatim)
	                      (arguments "(?f (multifieldp ?f))" "?type")
			      (body "(bind ?s \"\")
   (loop-for-count (?i (length$ ?f))
      (bind ?s (str-cat ?s (to-verbatim (nth$ ?i ?f) ?type))))
   ?s)")))

        ;; CLIPS
        (bind ?clipsGenericTemplate (assert (clips:template (name clips:generic))))
	(ignore (assert
	         (clips:template-slot (template ?clipsGenericTemplate) (name name))))
        (assert (clips:method (name to-verbatim)
		 (arguments "(?f FACT-ADDRESS (eq (fact-relation ?f) clips:generic))"
		  "(?type (eq ?type clips-source-code))")
	    (body "(format nil \"(defgeneric %s)%n\" (fact-slot-value ?f name)))")))
        (assert (clips:function (name clips-function-to-source-code-verbatim)
	                        (arguments "?name" "?f" "?type")
				(body "(bind ?s (format nil \"(def%s %s (\" ?name (fact-slot-value ?f name)))
    (bind ?args (fact-slot-value ?f arguments))
    (loop-for-count (?i (length$ ?args))
       (bind ?s (str-cat ?s (to-verbatim (nth$ ?i ?args) ?type) \" \")))
    (bind ?s (str-cat ?s (format nil \")%n%s)%n\" (to-verbatim (fact-slot-value ?f body) ?type))))
    ?s)")))

        (bind ?clipsMethodTemplate (assert (clips:template (name clips:method))))
	(ignore (assert
	         (clips:template-slot (template ?clipsMethodTemplate) (name name))
	         (clips:template-slot (template ?clipsMethodTemplate) (name arguments) (multi TRUE))
	         (clips:template-slot (template ?clipsMethodTemplate) (name body))))
        (assert (clips:method (name to-verbatim)
		 (arguments "(?f FACT-ADDRESS (eq (fact-relation ?f) clips:method))"
		  "(?type (eq ?type clips-source-code))")
	    (body "(clips-function-to-source-code-verbatim method ?f ?type)")))
        (bind ?clipsFunctionTemplate (assert (clips:template (name clips:function))))
	(ignore (assert
	         (clips:template-slot (template ?clipsFunctionTemplate) (name name))
	         (clips:template-slot (template ?clipsFunctionTemplate) (name arguments) (multi TRUE))
	         (clips:template-slot (template ?clipsFunctionTemplate) (name body))))
        (assert (clips:method (name to-verbatim)
		 (arguments "(?f FACT-ADDRESS (eq (fact-relation ?f) clips:function))"
		  "(?type (eq ?type clips-source-code))")
	    (body "(clips-function-to-source-code-verbatim function ?f ?type)")))
	(bind ?clipsRuleTemplate (assert (clips:template (name clips:rule))))
	(ignore (assert
		 (clips:template-slot (template ?clipsRuleTemplate) (name name))
	         (clips:template-slot (template ?clipsRuleTemplate) (name conditions) (multi TRUE))
	         (clips:template-slot (template ?clipsRuleTemplate) (name body))))
        (assert (clips:method (name to-verbatim)
(arguments "(?f FACT-ADDRESS (eq (fact-relation ?f) clips:rule))"
	   "(?type (eq ?type clips-source-code))")
(body "(bind ?s (format nil \"(defrule %s%n\" (fact-slot-value ?f name)))
    (bind ?conds (fact-slot-value ?f conditions))
    (loop-for-count (?i (length$ ?conds))
       (bind ?s (str-cat ?s (to-verbatim (nth$ ?i ?conds) ?type) (format nil \"%n\"))))
    (bind ?s (str-cat ?s (format nil \" =>%n %s)%n\" (to-verbatim (fact-slot-value ?f body) ?type))))
    ?s)")))
        (bind ?clipsTemplateTemplate (assert (clips:template (name clips:template))))
 	(ignore (assert
	         (clips:template-slot (template ?clipsTemplateTemplate) (name name))))
	(bind ?clipsTemplateSlotTemplate (assert (clips:template (name clips:template-slot))))
	(ignore (assert
	         (clips:template-slot (template ?clipsTemplateSlotTemplate) (name template))
	         (clips:template-slot (template ?clipsTemplateSlotTemplate) (name name))
		 (clips:template-slot (template ?clipsTemplateSlotTemplate) (name default))
	         (clips:template-slot (template ?clipsTemplateSlotTemplate) (name multi) (default FALSE))))
        (assert (clips:method (name to-verbatim)
	                      (arguments "(?f FACT-ADDRESS (eq (fact-relation ?f) clips:template))"
			                 "(?type (eq ?type clips-source-code))")
			      (body "(bind ?s (format nil \"(deftemplate %s %n\" (fact-slot-value ?f name)))
    (do-for-all-facts ((?slot clips:template-slot)) (eq ?slot:template ?f)
       (bind ?s (str-cat ?s (to-verbatim ?slot ?type))))
    (format nil \"%s)%n\" ?s))")))
        (assert (clips:method (name to-verbatim)
	                      (arguments "(?f FACT-ADDRESS (eq (fact-relation ?f) clips:template-slot))"
			            "(?type (eq ?type clips-source-code))")
			      (body "(bind ?multi (if (fact-slot-value ?f multi) then \"multi\" else \"\"))
(bind ?default (if (eq nil (fact-slot-value ?f default)) then \"\" else (format nil \" (default %s)\" (fact-slot-value ?f default)))) 
    (format nil \"(%sslot %s%s)%n\" ?multi (fact-slot-value ?f name) ?default))")))
        (bind ?clipsAssertionTemplate (assert (clips:template (name clips:assertion))))
	(ignore (assert (clips:template-slot (template ?clipsAssertionTemplate) (name is))))
	(assert (clips:method (name to-verbatim)
	                      (arguments "(?f FACT-ADDRESS (eq (fact-relation ?f) clips:assertion))"
			                 "(?type (eq ?type clips-source-code))")
			      (body "(format nil \"(assert %s)%n\" (to-verbatim (fact-slot-value ?f is) ?type))")))
	;; Files 
	(bind ?fileTemplate (assert (clips:template (name file))))
	(ignore (assert 
		 (clips:template-slot (template ?fileTemplate) (name name))
		 (clips:template-slot (template ?fileTemplate) (name is) (multi TRUE))
		))
	(bind ?fileTypeTemplate (assert (clips:template (name file-type))))
	(ignore (assert
	         (clips:template-slot (template ?fileTypeTemplate) (name file))
		 (clips:template-slot (template ?fileTypeTemplate) (name is))))
	(bind ?fileExtTypeMapping (assert (clips:template (name file-extension))))
	(ignore (assert
	          (clips:template-slot (template ?fileExtTypeMapping) (name is))
		  (clips:template-slot (template ?fileExtTypeMapping) (name assumed-type))))
	(assert (clips:assertion (is "(file-extension (is \"clp\") (assumed-type clips-source-code))")))
	(assert (clips:assertion (is "(file-extension (is \"md\") (assumed-type markdown))")))
        (assert (clips:rule (name file-type-mapping)
                (conditions "(file-extension (is ?ext) (assumed-type ?type))"
		            "?file <- (file (name ?name&:(str-endswithp ?name (str-cat \".\" ?ext))))"
                            "(not (exists (file-type (file ?file))))")
		(body "(assert (file-type (file ?file) (is ?type)))")))
        (bind ?fileContentTemplate (assert (clips:template (name file-content))))
	(ignore (assert
	         (clips:template-slot (template ?fileContentTemplate) (name file))
		 (clips:template-slot (template ?fileContentTemplate) (name is))))
        (assert (clips:rule (name produce-file-content) 
	                (conditions "?file <- (file (name ?name) (is $?data))"
                                    "(file-type (file ?file) (is ?type))")
			(body "(assert (file-content (file ?file) (is (to-verbatim ?data ?type))))")))
        (assert (clips:rule (name write-file)
                        (conditions "?file <- (file (name ?name))" "(file-content (file ?file) (is ?content))")
			(body "(bind ?logicalName (gensym))
    (open ?name ?logicalName \"wb\")
    (printout ?logicalName ?content)
    (close ?name)")))
        ;; Document
	(bind ?sectionTemplate (assert (clips:template (name section))))
	(ignore (assert
	         (clips:template-slot (template ?sectionTemplate) (name level) (default "1"))
		 (clips:template-slot (template ?sectionTemplate) (name title))
		 (clips:template-slot (template ?sectionTemplate) (name is) (multi TRUE))))
        (assert (clips:method (name to-verbatim)
	                      (arguments "(?f FACT-ADDRESS (and (eq (fact-relation ?f) section) (> (fact-slot-value ?f level) 0) (< (fact-slot-value ?f level) 7)))" 
			                 "(?type (eq ?type markdown))")
			      (body "(bind ?h \"\")
			      (loop-for-count (fact-slot-value ?f level)
			         (bind ?h (str-cat ?h \"#\")))
			      (format nil \"%s %s%n%n%s%n\" ?h (fact-slot-value ?f title) 
			         (to-verbatim (fact-slot-value ?f is) ?type))")))
       (bind ?olTemplate (assert (clips:template (name ol))))
       (ignore (assert (clips:template-slot (template ?olTemplate) (name is) (multi TRUE))))
       (assert (clips:method (name to-verbatim)
	                      (arguments "(?f FACT-ADDRESS (eq (fact-relation ?f) ol))"
			                 "(?type (eq ?type markdown))")
			      (body "(bind ?h \"\")
			      (bind ?is (fact-slot-value ?f is))
			      (loop-for-count (?i (length$ ?is)) 
			         (bind ?h (format nil \"%s* %s%n\" ?h (to-verbatim (nth$ ?i ?is) ?type))))
			      ?h")))
      (bind ?linkTemplate (assert (clips:template (name link))))
      (ignore (assert (clips:template-slot (template ?linkTemplate) (name name))
                      (clips:template-slot (template ?linkTemplate) (name to))))
       (assert (clips:method (name to-verbatim)
	                      (arguments "(?f FACT-ADDRESS (eq (fact-relation ?f) link))"
			                 "(?type (eq ?type markdown))")
			      (body "(format nil \"[%s](%s)\" (fact-slot-value ?f name) (fact-slot-value ?f to))")))

)))

(defglobal ?*quick-intro* =
   (create$
      "Conmacro (*construct macro*) is an exploration in how complex systems can be built.
Inspired by some of the original ideas behind literate programming, its based on the
idea of defining the entire program and its literature as abody of knowledge facts
(*constructs*) and using a rule production system (aka *expert system*) to infer
supplemental constructs & knowledge and produce verbatim files (*macro expansion*)

"))

(assert (file (name "README.md")
              (is 

(assert (section (title "Conmacro")
	 (is ?*quick-intro*)))
(assert (section (level 2) (title "Requirements")
(is
(assert (ol (is
(assert (link (to "http://clipsrules.net") (name "CLIPS 6.30")))
)))
)))

(assert (section (level 2) (title "Using")
(is
"In order to invoke conmacro, run

```shell
clips -f2 conmacro_boot.clp <file.clp>
```
"
)))

)))

(run)
(exit)
