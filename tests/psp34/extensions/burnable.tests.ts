import { bnArg, expect, setupContract, fromSigner } from '../../helpers'

describe('MY_PSP34_BURNABLE', () => {
    async function setup() {
        return setupContract('my_psp34_burnable', 'new')
    }

    it('Burn works', async () => {
        const {
            contract,
            defaultSigner: sender,
            query
        } = await setup()

        await expect(query.balanceOf(sender.address)).to.have.output(3)

        await contract.tx.burn(bnArg(0, 1))

        await expect(query.balanceOf(sender.address)).to.have.output(2)
    })

    it('Burn from works', async () => {
        const {
            contract,
            defaultSigner: sender,
            accounts: [alice],
            query
        } = await setup()

        await expect(query.balanceOf(sender.address)).to.have.output(3)
        await contract.tx.setApprovalForAll(alice.address, true)

        await fromSigner(contract, alice.address).tx.burnFrom(sender.address, bnArg(0, 1))

        await expect(query.balanceOf(sender.address)).to.have.output(2)
    })

    it('Burn from without allowance should fail', async () => {
        const {
            contract,
            defaultSigner: sender,
            accounts: [alice],
            query
        } = await setup()

        await expect(query.balanceOf(sender.address)).to.have.output(3)

        await expect(fromSigner(contract, alice.address).tx.burnFrom(sender.address, bnArg(0, 1)))
            .to.eventually.be.rejected

        await expect(query.balanceOf(sender.address)).to.have.output(3)
    })

})
