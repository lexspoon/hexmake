#include "lib.h"

#include <stdio.h>
#include <stdlib.h>

int main(int argc, char **argv) 
{
  if (argc != 3) 
    {
      printf("Usage: main a b\n");
      return 1;
    }


  int a = atoi(argv[1]);
  int b = atoi(argv[2]);
  int answer = sum(a, b);

  printf("Sum: %d\n", answer);

  return 0;
}
