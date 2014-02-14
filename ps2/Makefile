PROGRAM_NAME = gash

all: $(PROGRAM_NAME)

$(PROGRAM_NAME): $(PROGRAM_NAME).rs
	rustc $(PROGRAM_NAME).rs

clean :
	$(RM) $(PROGRAM_NAME)
    
run: ${PROGRAM_NAME}
	./${PROGRAM_NAME}
