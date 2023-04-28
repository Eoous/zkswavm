use halo2_proofs::plonk::Error;
use halo2_proofs::{
    arithmetic::FieldExt,
    plonk::{Advice, Column, ConstraintSystem, Expression, VirtualCells},
};
use num_bigint::BigUint;
use specs::etable::EventTableEntry;
use specs::itable::{OpcodeClass, OPCODE_CLASS_SHIFT, OPCODE_VTYPE_SHIFT};
use specs::mtable::VarType;
use specs::step::StepInfo;
use std::marker::PhantomData;

use crate::circuits::event::EventCommonConfig;
use crate::circuits::event::{EventOpcodeConfig, EventOpcodeConfigBuilder};
use crate::circuits::instruction::InstructionConfig;
use crate::circuits::jump::JumpConfig;
use crate::circuits::memory::MemoryConfig;
use crate::circuits::range::RangeConfig;
use crate::circuits::utils::tvalue::TValueConfig;
use crate::circuits::utils::{bn_to_field, Context};
use crate::{constant, constant_from, cur};

pub struct ConstConfig<F: FieldExt> {
    tvalue: TValueConfig<F>,
    enable: Column<Advice>,
}

pub struct ConstConfigBuilder {}

impl<F: FieldExt> EventOpcodeConfigBuilder<F> for ConstConfigBuilder {
    fn configure(
        meta: &mut ConstraintSystem<F>,
        common: &EventCommonConfig,
        opcode_bit: Column<Advice>,
        cols: &mut impl Iterator<Item = Column<Advice>>,
        range_table: &RangeConfig<F>,
        instruction_table: &InstructionConfig<F>,
        memory_table: &MemoryConfig<F>,
        jump_table: &JumpConfig<F>,
    ) -> Box<dyn EventOpcodeConfig<F>> {
        let tvalue =
            TValueConfig::configure(meta, cols, range_table, |meta| cur!(meta, opcode_bit));

        memory_table.configure_stack_write_in_table(
            "const mlookup",
            meta,
            |meta| cur!(meta, opcode_bit),
            |meta| cur!(meta, common.eid),
            |meta| constant_from!(1u64),
            |meta| cur!(meta, common.sp),
            |meta| cur!(meta, tvalue.vtype),
            |meta| cur!(meta, tvalue.value.value),
        );

        Box::new(ConstConfig {
            enable: opcode_bit,
            tvalue,
        })
    }
}

impl<F: FieldExt> EventOpcodeConfig<F> for ConstConfig<F> {
    fn opcode(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F> {
        // [(2 + vartype) << (64+13) + value] * enable
        // 2(10) << 77
        (constant!(bn_to_field(&(BigUint::from(OpcodeClass::Const as u64) << OPCODE_CLASS_SHIFT)))
            // vartype * (1 << 77)
            + cur!(meta, self.tvalue.vtype) * constant!(bn_to_field(&(BigUint::from(1u64) << OPCODE_VTYPE_SHIFT)))
            // value
            + cur!(meta, self.tvalue.value.value))
            * cur!(meta, self.enable)
    }

    fn sp_diff(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F> {
        // 1 * enable
        // 0 || 1
        constant_from!(1u64) * cur!(meta, self.enable)
    }

    fn assign(&self, ctx: &mut Context<'_, F>, entry: &EventTableEntry) -> Result<(), Error> {
        match entry.step_info {
            StepInfo::I32Const { value } => {
                self.tvalue.assign(ctx, VarType::I32, value as u32 as u64)?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn opcode_class(&self) -> OpcodeClass {
        OpcodeClass::Const
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime::{WasmInterpreter, WasmRuntime};
    use halo2_proofs::pairing::bn256::Fr as Fp;
    use wasmi::{ImportsBuilder, ModuleInstance};

    use crate::test::test_circuit_builder::run_test_circuit;

    #[test]
    fn test_ok() {
        let textual_repr = r#"
                (module
                    (func (export "test")
                      (i32.const 0)
                      (drop)
                    )
                   )
                "#;

        let compiler = WasmInterpreter::new();
        let compiled_module = compiler.compile(textual_repr).unwrap();
        let execution_log = compiler.run(&compiled_module, "test", vec![]).unwrap();

        run_test_circuit::<Fp>(compiled_module.tables, execution_log.tables).unwrap()
    }
}
