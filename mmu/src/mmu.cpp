// trace flags and macros
bool O_option;
bool P_option;
bool F_option;
bool S_option;
bool x_option;
bool y_option;
bool f_option;
bool a_option;

#define O_trace(fmt...) do { if (O_option) { printf(fmt); printf("\n"); fflush(stdout); } } while(0)
#define P_trace(fmt...) do { if (P_option) { printf(fmt); printf("\n"); fflush(stdout); } } while(0)
#define F_trace(fmt...) do { if (F_option) { printf(fmt); printf("\n"); fflush(stdout); } } while(0)
#define S_trace(fmt...) do { if (S_option) { printf(fmt); printf("\n"); fflush(stdout); } } while(0)
#define x_trace(fmt...) do { if (x_option) { printf(fmt); printf("\n"); fflush(stdout); } } while(0)
#define y_trace(fmt...) do { if (y_option) { printf(fmt); printf("\n"); fflush(stdout); } } while(0)
#define f_trace(fmt...) do { if (f_option) { printf(fmt); printf("\n"); fflush(stdout); } } while(0)
#define a_trace(fmt...) do { if (a_option) { printf(fmt); printf("\n"); fflush(stdout); } } while(0)

