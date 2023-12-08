
// ==================== main.c ==================== //
// Follow this file template to implement your game //
// ================================================ //

#define SIM 1 // change value depending if running simulation or not

// ================================ //
#pragma code-name ("CODE")
#pragma bss-name ("BSS")

#include <int.h>
#include <vram.h>
#include <stop.h>
#include <Q9_6.h>
#include <mapache64_zero_page.h>
#include <controller.h>
#include <screen.h>



// expects int n > 0
int recursiveFibo(int n) {
    if (n <= 1) {
        return n;
    } else {
        return recursiveFibo(n - 1) + recursiveFibo(n - 2);
    }
}
//expects int n > 0
short int iterativeFibo(int n) {
    int first;
    int second;
    int next;
    int i;
    if (n < 0) {
        return -1;
    }
    first = 0;
    second = 1;
    //++i for good cc65 code search '++' here https://cc65.github.io/doc/coding.html 
    for (i = 0; i < n + 1; ++i) {
        if (i <= 1) {
            next = i;
        } else {
            next = first + second;
            first = second;
            second = next;
        }
    }

    return next;
}


// run once on startup
void reset(void) {
    // int recur, iter;
    short int *p = (short int *)0x0200;
    // Use malloc to allocate memory at the desired address
    int i;
    // recur = recursiveFibo(4);
    // recur = 15000;
    for (i = 0; i < 10; ++i) {
        *(p + i) = iterativeFibo(i); 
    }



    __asm__("BRK");

    // iter = iterativeFibo(10);
    // __asm__("BRK");
} 


// run 60 times a second
void do_logic(void) { }

// run after do_logic and once gpu is idle
void fill_vram(void) { }
