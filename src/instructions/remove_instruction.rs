use std::{iter::Peekable, slice::Iter};

use super::{
    InstructionBytes, InstructionContent, InstructionError, InstructionInfo, Result,
    REMOVE_INSTRUCTION_SIGN,
};

#[derive(Debug, PartialEq, Clone)]
pub struct RemoveInstruction {
    length: u8,
}

impl RemoveInstruction {
    pub fn new(length: u8) -> Self {
        Self { length }
    }
}

impl InstructionInfo for RemoveInstruction {
    fn len(&self) -> u8 {
        self.length
    }

    fn is_empty(&self) -> bool {
        self.len() == u8::MIN
    }

    fn is_full(&self) -> bool {
        self.len() == u8::MAX
    }

    fn non_default_item_count(&self) -> Option<u8> {
        None
    }
}

impl InstructionContent for RemoveInstruction {
    fn push(&mut self, _: u8) -> Result<()> {
        if self.is_full() {
            return Err(InstructionError::ContentOverflow);
        }
        self.length += 1;
        Ok(())
    }

    fn fill(
        &mut self,
        lcs: &mut Peekable<Iter<'_, u8>>,
        source: &mut Peekable<Iter<'_, u8>>,
        _: &mut Peekable<Iter<'_, u8>>,
    ) {
        while source.peek().is_some() && lcs.peek() != source.peek() && !self.is_full() {
            self.push(*source.next().unwrap()).unwrap();
        }
    }

    fn apply(&self, source: &mut Iter<'_, u8>, _: &mut Vec<u8>) {
        for _ in 0..self.len() {
            source.next();
        }
    }
}

impl InstructionBytes for RemoveInstruction {
    fn byte_sign(&self) -> u8 {
        REMOVE_INSTRUCTION_SIGN
    }

    fn byte_length(&self) -> usize {
        2
    }

    fn to_bytes(&self) -> Vec<u8> {
        vec![self.byte_sign(), self.len()]
    }

    fn try_from_bytes(bytes: &mut Peekable<Iter<'_, u8>>) -> Result<Self> {
        match bytes.next() {
            Some(&REMOVE_INSTRUCTION_SIGN) => (),
            Some(_) => return Err(InstructionError::InvalidSign),
            None => return Err(InstructionError::MissignSign),
        };

        if bytes.peek().is_none() {
            return Err(InstructionError::MissingLength);
        }

        let length = *bytes.next().ok_or(InstructionError::MissingLength)?;
        Ok(Self { length })
    }
}

impl Default for RemoveInstruction {
    fn default() -> Self {
        Self::new(u8::MIN)
    }
}

impl From<&RemoveInstruction> for Vec<u8> {
    fn from(value: &RemoveInstruction) -> Self {
        value.to_bytes()
    }
}

impl From<RemoveInstruction> for Vec<u8> {
    fn from(value: RemoveInstruction) -> Self {
        value.to_bytes()
    }
}

impl TryFrom<&mut Peekable<Iter<'_, u8>>> for RemoveInstruction {
    type Error = InstructionError;

    fn try_from(value: &mut Peekable<Iter<'_, u8>>) -> std::result::Result<Self, Self::Error> {
        RemoveInstruction::try_from_bytes(value)
    }
}

impl TryFrom<Peekable<Iter<'_, u8>>> for RemoveInstruction {
    type Error = InstructionError;

    fn try_from(mut value: Peekable<Iter<'_, u8>>) -> std::result::Result<Self, Self::Error> {
        RemoveInstruction::try_from_bytes(&mut value)
    }
}

impl TryFrom<Vec<u8>> for RemoveInstruction {
    type Error = InstructionError;

    fn try_from(value: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        RemoveInstruction::try_from_bytes(&mut value.iter().peekable())
    }
}

impl TryFrom<&[u8]> for RemoveInstruction {
    type Error = InstructionError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        RemoveInstruction::try_from_bytes(&mut value.iter().peekable())
    }
}

#[cfg(test)]
mod remove_instruction_tests {
    use crate::lcs::Lcs;

    use super::*;

    #[test]
    fn instruction_info() {
        let mut instruction = RemoveInstruction::new(u8::MAX);
        assert_eq!(instruction.len(), u8::MAX);
        assert!(instruction.is_full());

        instruction = RemoveInstruction::new(u8::MIN);
        assert_eq!(instruction.len(), u8::MIN);
        assert!(instruction.is_empty());

        let default_instruction = RemoveInstruction::default();
        assert_eq!(default_instruction, instruction);
    }

    #[test]
    fn instruction_content_push() {
        let mut instruction = RemoveInstruction::new(u8::MAX - 1);
        assert!(instruction.push(0).is_ok());
        assert_eq!(instruction.push(0), Err(InstructionError::ContentOverflow));
    }

    fn fill_wrapper(source: &[u8], target: &[u8]) -> RemoveInstruction {
        let mut instruction = RemoveInstruction::default();
        let lcs = Lcs::new(source, target).subsequence();
        let mut lcs_iter = lcs.iter().peekable();
        let mut source_iter = source.iter().peekable();
        let mut target_iter = target.iter().peekable();
        instruction.fill(&mut lcs_iter, &mut source_iter, &mut target_iter);
        instruction
    }

    #[test]
    fn instruction_content_fill() {
        let instruction = fill_wrapper(b"AAA", b"");
        assert_eq!(instruction.len(), 3);
        let instruction = fill_wrapper(b"AAA", b"B");
        assert_eq!(instruction.len(), 3);
        let instruction = fill_wrapper(b"AAA", b"BBA");
        assert_eq!(instruction.len(), 0);
    }

    #[test]
    fn instruction_bytes_to_bytes() {
        let mut instruction = RemoveInstruction::new(u8::MAX);
        let mut bytes = vec![REMOVE_INSTRUCTION_SIGN];
        bytes.extend(instruction.len().to_be_bytes());
        assert_eq!(instruction.to_bytes(), bytes);

        instruction = RemoveInstruction::default();
        bytes = vec![REMOVE_INSTRUCTION_SIGN];
        bytes.extend(instruction.len().to_be_bytes());
        assert_eq!(instruction.to_bytes(), bytes);
    }

    #[test]
    fn instruction_bytes_try_from_bytes_ok() {
        let mut instruction = RemoveInstruction::new(u8::MAX);
        let mut bytes = instruction.to_bytes();
        assert_eq!(
            RemoveInstruction::try_from_bytes(&mut bytes.iter().peekable()),
            Ok(instruction)
        );

        instruction = RemoveInstruction::default();
        bytes = instruction.to_bytes();
        assert_eq!(
            RemoveInstruction::try_from_bytes(&mut bytes.iter().peekable()),
            Ok(instruction)
        );
    }

    #[test]
    fn instruction_bytes_try_from_bytes_err() {
        let mut bytes: Vec<u8> = vec![];
        assert_eq!(
            RemoveInstruction::try_from_bytes(&mut bytes.iter().peekable()),
            Err(InstructionError::MissignSign)
        );
        bytes = vec![b'A'];
        assert_eq!(
            RemoveInstruction::try_from_bytes(&mut bytes.iter().peekable()),
            Err(InstructionError::InvalidSign)
        );
        bytes = vec![REMOVE_INSTRUCTION_SIGN];
        assert_eq!(
            RemoveInstruction::try_from_bytes(&mut bytes.iter().peekable()),
            Err(InstructionError::MissingLength)
        );
    }
}
