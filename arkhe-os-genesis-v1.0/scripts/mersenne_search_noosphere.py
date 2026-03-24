# mersenne_search_noosphere.py
import sympy
from sympy.ntheory import isprime

def mersenne_candidates(limit=50):
    """Gera candidatos a primos de Mersenne até o limite de expoente."""
    primes = []
    for p in range(2, limit+1):
        if isprime(p):
            m = 2**p - 1
            if isprime(m):
                primes.append(p)
    return primes

def perfect_numbers_from_mersenne(primes):
    """Retorna os números perfeitos correspondentes."""
    return [(p, 2**(p-1) * (2**p - 1)) for p in primes]

if __name__ == "__main__":
    # Os já conhecidos até p=31
    conhecidos = mersenne_candidates(31)
    perfeitos = perfect_numbers_from_mersenne(conhecidos)

    print("Primos de Mersenne conhecidos (p <= 31):", conhecidos)
    print("Números perfeitos correspondentes:")
    for p, perf in perfeitos:
        print(f"p={p:2d} -> n={2*p:2d} -> perfeito = {perf}")

    # Simular a busca por novos na Noosfera
    print("\n🔮 A Noosfera está sondando...")
    # Em um sistema real, a Noosfera usaria padrões de ressonância para detectar
    # novos primos. Aqui, apenas listamos os próximos candidatos.
    proximos = [37, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97]
    print("Candidatos naturais (expoentes primos seguintes):", proximos)
