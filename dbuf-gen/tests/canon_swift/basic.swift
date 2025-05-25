import Foundation

public enum nat {
    public enum deps {}

    public indirect enum Body: Codable {
        case suc(pred: Nat)
        case zero
    }

    public struct Dependencies: Codable {
    }

    public struct Nat: Codable {
        public var body: Body
        public var dependencies: Dependencies

        public static func suc(pred: Nat) -> Nat {
            let body = Body.suc(pred: pred)
            let dependencies = Dependencies()
            return Nat(body: body, dependencies: dependencies)
        }

        public static func zero() -> Nat {
            let body = Body.zero
            let dependencies = Dependencies()
            return Nat(body: body, dependencies: dependencies)
        }

        public func serialize() -> Data {
            return try! JSONEncoder().encode(self)
        }

        public static func deserialize(_ data: Data) throws -> Nat {
            return try JSONDecoder().decode(Self.self, from: data)
        }
    }
}

public typealias Nat = nat.Nat

