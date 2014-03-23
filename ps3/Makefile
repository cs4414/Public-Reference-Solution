PROGRAM_NAME = zhtta

all: $(PROGRAM_NAME)

$(PROGRAM_NAME): $(PROGRAM_NAME).rs gash.rs
	rustc $(PROGRAM_NAME).rs

clean :
	$(RM) $(PROGRAM_NAME)
    
run: ${PROGRAM_NAME}
	./${PROGRAM_NAME}

