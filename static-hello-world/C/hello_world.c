#include <stdio.h>
#include <stdint.h>

typedef struct aux_vec {
    uint64_t key;
    uint64_t val;
} aux_vec_t;

enum aux_var_type {
    // ### architecture neutral
    /// end of vector
    AtNull = 0,
    /// entry should be ignored
    AtIgnore = 1,
    /// file descriptor of program
    AtExecfd = 2,
    /// program headers for program
    AtPhdr = 3,
    /// size of program header entry
    AtPhent = 4,
    /// number of program headers
    AtPhnum = 5,
    /// system page size
    AtPagesz = 6,
    /// The base address of the program interpreter (usually, the
    /// dynamic linker).
    AtBase = 7,
    /// flags
    AtFlags = 8,
    /// entry point of program
    AtEntry = 9,
    /// program is not ELF
    AtNotelf = 10,
    /// real uid
    AtUid = 11,
    /// effective uid
    AtEuid = 12,
    /// real gid
    AtGid = 13,
    /// effective gid
    AtEgid = 14,
    /// string identifying CPU for optimizations
    AtPlatform = 15,
    /// arch dependent hints at CPU capabilities
    AtHwcap = 16,
    /// frequency at which times() increments
    AtClktck = 17,
    /// secure mode boolean
    AtSecure = 23,
    /// string identifying real platform, may differ from AtPlatform.
    AtBasePlatform = 24,
    /// address of 16 random bytes
    AtRandom = 25,
    /// extension of AtHwcap
    AtHwcap2 = 26,
    /// filename of program, for example "./my_executable\0"
    AtExecFn = 31,

    // ### according to Linux code: from here: x86_64
    /// The entry point to the system call function in the vDSO.
    /// Not present/needed on all architectures (e.g., absent on
    /// x86-64).
    AtSysinfo = 32,
    /// The address of a page containing the virtual Dynamic
    /// Shared Object (vDSO) that the kernel creates in order to
    /// provide fast implementations of certain system calls.
    AtSysinfoEhdr = 33,

    // ### according to Linux code: from here: PowerPC
    AtL1iCachesize = 40,
    AtL1iCachegeometry = 41,
    AtL1dCachesize = 42,
    AtL1dCachegeometry = 43,
    AtL2Cachesize = 44,
    AtL2Cachegeometry = 45,
    AtL3Cachesize = 46,
    AtL3Cachegeometry = 47,
};

char * aux_var_type_to_str(uint64_t type) {
    switch (type)
    {
    case AtNull: return "AtNull";
    case AtIgnore: return "AtIgnore";
    case AtExecfd: return "AtExecfd";
    case AtPhdr: return "AtPhdr";
    case AtPhent: return "AtPhent";
    case AtPhnum: return "AtPhnum";
    case AtPagesz: return "AtPagesz";
    case AtBase: return "AtBase";
    case AtFlags: return "AtFlags";
    case AtEntry: return "AtEntry";
    case AtNotelf: return "AtNotelf";
    case AtUid: return "AtUid";
    case AtEuid: return "AtEuid";
    case AtGid: return "AtGid";
    case AtEgid: return "AtEgid";
    case AtPlatform: return "AtPlatform";
    case AtHwcap: return "AtHwcap";
    case AtClktck: return "AtClktck";
    case AtSecure: return "AtSecure";
    case AtBasePlatform: return "AtBasePlatform";
    case AtRandom: return "AtRandom";
    case AtHwcap2: return "AtHwcap2";
    case AtExecFn: return "AtExecFn";
    case AtSysinfo: return "AtSysinfo";
    case AtSysinfoEhdr: return "AtSysinfoEhdr";
    default:
        return "Unknown";
    }
}

int main(int argc, char *argv[], char * envp[]) {
	printf("hello world from linux\n");

    printf("there are %d args\n", argc);
    for (char** ptr = argv; *ptr; ptr++) {
        printf("  %s\n", *ptr);
    }


    int envpc = 0;
    for (char** ptr = envp; *ptr; ptr++) {
        envpc++;
    }

    printf("there are %d env vars\n", envpc);
    for (char** ptr = envp; *ptr; ptr++) {
        printf("  %s\n", *ptr);
    }

    printf("AT-Values / auxiliary vector\n");
    aux_vec_t * aux = (aux_vec_t *) (envp + envpc + 1);
    printf("envp: %p\n", envp);
    printf("aux : %p\n", aux);
    for (; aux->key; aux++) {
        printf("  %p: %s(%ld) => %lx\n", aux, aux_var_type_to_str(aux->key), aux->key, aux->val);
    }
    printf("  %p: %s(%ld) => %lx\n", aux, aux_var_type_to_str(AtNull), (uint64_t) AtNull, (uint64_t) 0);
    
    /*for (int i = 1; i < AtSysinfoEhdr; i++) {
        printf("  %s(%ld) => %ld\n", aux_var_type_to_str(i), i, getauxval(i));
    }*/
}
